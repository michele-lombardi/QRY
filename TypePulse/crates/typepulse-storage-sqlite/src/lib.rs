//! SQLite persistence adapter for aggregate TypePulse statistics.
//!
//! The schema cannot represent key identity, text, application names, window
//! titles, or individual event timestamps.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use typepulse_core::{
    AppPreferences, CompletedSessionRecord, DailySummary, LocalDate, MetricBucketRecord,
    RepositoryError, RepositoryErrorKind, StatisticsRepository,
};

const LATEST_SCHEMA_VERSION: usize = 1;
const MIGRATION_LIST: &[M<'_>] = &[M::up(include_str!("../migrations/0001_initial.sql"))];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATION_LIST);

/// SQLite implementation of the aggregate statistics repository.
pub struct SqliteStatisticsRepository {
    connection: Connection,
    database_path: Option<PathBuf>,
    last_backup_path: Option<PathBuf>,
}

impl SqliteStatisticsRepository {
    /// Opens or creates an on-disk database and applies pending migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RepositoryError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                repository_error(RepositoryErrorKind::Io, "create database directory", error)
            })?;
        }

        let mut connection = Connection::open(path).map_err(|error| {
            repository_error(RepositoryErrorKind::Open, "open SQLite database", error)
        })?;
        let current_version = schema_version(&connection)?;
        let last_backup_path = if current_version < LATEST_SCHEMA_VERSION
            && fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0)
        {
            connection
                .execute_batch("PRAGMA wal_checkpoint(FULL);")
                .map_err(|error| {
                    repository_error(RepositoryErrorKind::Io, "checkpoint before backup", error)
                })?;
            drop(connection);
            let backup_path = migration_backup_path(path, current_version);
            fs::copy(path, &backup_path).map_err(|error| {
                repository_error(
                    RepositoryErrorKind::Io,
                    "create pre-migration backup",
                    error,
                )
            })?;
            connection = Connection::open(path).map_err(|error| {
                repository_error(RepositoryErrorKind::Open, "reopen SQLite database", error)
            })?;
            Some(backup_path)
        } else {
            None
        };

        migrate(&mut connection)?;
        configure_connection(&connection, true)?;
        Ok(Self {
            connection,
            database_path: Some(path.to_path_buf()),
            last_backup_path,
        })
    }

    /// Creates a migrated in-memory database for deterministic tests.
    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        let mut connection = Connection::open_in_memory().map_err(|error| {
            repository_error(RepositoryErrorKind::Open, "open in-memory SQLite", error)
        })?;
        migrate(&mut connection)?;
        configure_connection(&connection, false)?;
        Ok(Self {
            connection,
            database_path: None,
            last_backup_path: None,
        })
    }

    /// Current `PRAGMA user_version` after migrations.
    pub fn schema_version(&self) -> Result<usize, RepositoryError> {
        schema_version(&self.connection)
    }

    /// On-disk path, absent for an in-memory repository.
    #[must_use]
    pub fn database_path(&self) -> Option<&Path> {
        self.database_path.as_deref()
    }

    /// Backup created immediately before the latest migration, when applicable.
    #[must_use]
    pub fn last_backup_path(&self) -> Option<&Path> {
        self.last_backup_path.as_deref()
    }

    /// Exposes the connection for aggregate-only diagnostics and schema audits.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

impl StatisticsRepository for SqliteStatisticsRepository {
    fn save_session(&mut self, session: CompletedSessionRecord) -> Result<(), RepositoryError> {
        validate_session(session)?;
        self.connection
            .execute(
                "INSERT OR IGNORE INTO completed_sessions (
                    local_date, started_at_unix_ms, ended_at_unix_ms,
                    estimated_character_count, estimated_word_count, average_wpm,
                    peak_wpm, active_typing_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    session.local_date.to_string(),
                    session.started_at_unix_ms,
                    session.ended_at_unix_ms,
                    to_i64(session.estimated_character_count)?,
                    session.estimated_word_count,
                    session.average_wpm,
                    session.peak_wpm,
                    duration_to_i64_ms(session.active_typing_duration)?,
                ],
            )
            .map_err(query_error)?;
        Ok(())
    }

    fn save_bucket(&mut self, bucket: MetricBucketRecord) -> Result<(), RepositoryError> {
        validate_bucket(bucket)?;
        self.connection
            .execute(
                "INSERT INTO metric_buckets (
                    local_date, interval_start_unix_ms, interval_duration_ms,
                    estimated_character_count, average_wpm, peak_wpm
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(interval_start_unix_ms) DO UPDATE SET
                    local_date = excluded.local_date,
                    interval_duration_ms = excluded.interval_duration_ms,
                    average_wpm = CASE
                        WHEN metric_buckets.estimated_character_count
                           + excluded.estimated_character_count = 0 THEN 0
                        ELSE (
                            metric_buckets.average_wpm
                              * metric_buckets.estimated_character_count
                            + excluded.average_wpm
                              * excluded.estimated_character_count
                        ) / (
                            metric_buckets.estimated_character_count
                            + excluded.estimated_character_count
                        )
                    END,
                    peak_wpm = MAX(metric_buckets.peak_wpm, excluded.peak_wpm),
                    estimated_character_count =
                        metric_buckets.estimated_character_count
                        + excluded.estimated_character_count",
                params![
                    bucket.local_date.to_string(),
                    bucket.interval_start_unix_ms,
                    duration_to_i64_ms(bucket.interval_duration)?,
                    to_i64(bucket.estimated_character_count)?,
                    bucket.average_wpm,
                    bucket.peak_wpm,
                ],
            )
            .map_err(query_error)?;
        Ok(())
    }

    fn daily_summary(&mut self, date: LocalDate) -> Result<DailySummary, RepositoryError> {
        let values = self
            .connection
            .query_row(
                "SELECT
                    COALESCE(SUM(estimated_character_count), 0),
                    COALESCE(SUM(estimated_word_count), 0.0),
                    CASE WHEN COALESCE(SUM(estimated_character_count), 0) = 0 THEN 0.0
                         ELSE SUM(average_wpm * estimated_character_count)
                              / SUM(estimated_character_count) END,
                    COALESCE(MAX(peak_wpm), 0.0),
                    COALESCE(SUM(active_typing_ms), 0),
                    COUNT(*)
                 FROM completed_sessions WHERE local_date = ?1",
                [date.to_string()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .map_err(query_error)?;
        summary_from_values(date, values)
    }

    fn recent_daily_summaries(
        &mut self,
        through: LocalDate,
        day_count: usize,
    ) -> Result<Vec<DailySummary>, RepositoryError> {
        let mut dates = Vec::with_capacity(day_count);
        let mut cursor = Some(through);
        for _ in 0..day_count {
            let Some(date) = cursor else {
                break;
            };
            dates.push(date);
            cursor = date.previous_day();
        }
        dates.reverse();
        dates
            .into_iter()
            .map(|date| self.daily_summary(date))
            .collect()
    }

    fn metric_buckets(
        &mut self,
        date: LocalDate,
    ) -> Result<Vec<MetricBucketRecord>, RepositoryError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT interval_start_unix_ms, interval_duration_ms,
                        estimated_character_count, average_wpm, peak_wpm
                 FROM metric_buckets WHERE local_date = ?1
                 ORDER BY interval_start_unix_ms",
            )
            .map_err(query_error)?;
        let rows = statement
            .query_map([date.to_string()], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            })
            .map_err(query_error)?;
        let mut buckets = Vec::new();
        for row in rows {
            let (start, duration_ms, characters, average_wpm, peak_wpm) =
                row.map_err(query_error)?;
            buckets.push(MetricBucketRecord {
                local_date: date,
                interval_start_unix_ms: start,
                interval_duration: duration_from_i64_ms(duration_ms)?,
                estimated_character_count: to_u64(characters)?,
                average_wpm: valid_metric(average_wpm)?,
                peak_wpm: valid_metric(peak_wpm)?,
            });
        }
        Ok(buckets)
    }

    fn reset_day(&mut self, date: LocalDate) -> Result<(), RepositoryError> {
        let transaction = self.connection.transaction().map_err(query_error)?;
        transaction
            .execute(
                "DELETE FROM completed_sessions WHERE local_date = ?1",
                [date.to_string()],
            )
            .map_err(query_error)?;
        transaction
            .execute(
                "DELETE FROM metric_buckets WHERE local_date = ?1",
                [date.to_string()],
            )
            .map_err(query_error)?;
        transaction.commit().map_err(query_error)
    }

    fn load_preferences(&mut self) -> Result<AppPreferences, RepositoryError> {
        let enabled = self
            .connection
            .query_row(
                "SELECT auto_start_enabled FROM app_preferences WHERE singleton_id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(query_error)?
            .unwrap_or(0);
        match enabled {
            0 => Ok(AppPreferences {
                auto_start_enabled: false,
            }),
            1 => Ok(AppPreferences {
                auto_start_enabled: true,
            }),
            _ => Err(invalid_data("invalid auto-start preference")),
        }
    }

    fn save_preferences(&mut self, preferences: AppPreferences) -> Result<(), RepositoryError> {
        self.connection
            .execute(
                "INSERT INTO app_preferences(singleton_id, auto_start_enabled) VALUES (1, ?1)
                 ON CONFLICT(singleton_id) DO UPDATE SET
                    auto_start_enabled = excluded.auto_start_enabled",
                [i64::from(preferences.auto_start_enabled)],
            )
            .map_err(query_error)?;
        Ok(())
    }
}

fn migrate(connection: &mut Connection) -> Result<(), RepositoryError> {
    MIGRATIONS.to_latest(connection).map_err(|error| {
        repository_error(
            RepositoryErrorKind::Migration,
            "apply SQLite migrations",
            error,
        )
    })
}

fn configure_connection(connection: &Connection, on_disk: bool) -> Result<(), RepositoryError> {
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(query_error)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(query_error)?;
    if on_disk {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(query_error)?;
        connection
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(query_error)?;
    }
    Ok(())
}

fn schema_version(connection: &Connection) -> Result<usize, RepositoryError> {
    let version = connection
        .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .map_err(query_error)?;
    usize::try_from(version).map_err(|_| invalid_data("negative SQLite schema version"))
}

fn migration_backup_path(path: &Path, version: usize) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("typepulse.sqlite3");
    path.with_file_name(format!(
        "{file_name}.pre-migration-v{version}-{timestamp}.bak"
    ))
}

fn validate_session(session: CompletedSessionRecord) -> Result<(), RepositoryError> {
    if session.ended_at_unix_ms < session.started_at_unix_ms
        || !valid_nonnegative(session.estimated_word_count)
        || !valid_nonnegative(session.average_wpm)
        || !valid_nonnegative(session.peak_wpm)
    {
        return Err(invalid_data("invalid completed session aggregate"));
    }
    Ok(())
}

fn validate_bucket(bucket: MetricBucketRecord) -> Result<(), RepositoryError> {
    if bucket.interval_duration.is_zero()
        || !valid_nonnegative(bucket.average_wpm)
        || !valid_nonnegative(bucket.peak_wpm)
    {
        return Err(invalid_data("invalid metric bucket aggregate"));
    }
    Ok(())
}

fn summary_from_values(
    date: LocalDate,
    values: (i64, f64, f64, f64, i64, i64),
) -> Result<DailySummary, RepositoryError> {
    Ok(DailySummary {
        date,
        estimated_character_count: to_u64(values.0)?,
        estimated_word_count: valid_metric(values.1)?,
        average_wpm: valid_metric(values.2)?,
        peak_wpm: valid_metric(values.3)?,
        active_typing_duration: duration_from_i64_ms(values.4)?,
        session_count: to_u64(values.5)?,
    })
}

fn valid_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn valid_metric(value: f64) -> Result<f64, RepositoryError> {
    if valid_nonnegative(value) {
        Ok(value)
    } else {
        Err(invalid_data("stored metric is negative or non-finite"))
    }
}

fn to_i64(value: u64) -> Result<i64, RepositoryError> {
    i64::try_from(value).map_err(|_| invalid_data("aggregate exceeds SQLite integer range"))
}

fn to_u64(value: i64) -> Result<u64, RepositoryError> {
    u64::try_from(value).map_err(|_| invalid_data("stored aggregate is negative"))
}

fn duration_to_i64_ms(duration: Duration) -> Result<i64, RepositoryError> {
    i64::try_from(duration.as_millis())
        .map_err(|_| invalid_data("duration exceeds SQLite integer range"))
}

fn duration_from_i64_ms(value: i64) -> Result<Duration, RepositoryError> {
    Ok(Duration::from_millis(to_u64(value)?))
}

fn query_error(error: rusqlite::Error) -> RepositoryError {
    repository_error(RepositoryErrorKind::Query, "SQLite query", error)
}

fn invalid_data(message: &str) -> RepositoryError {
    RepositoryError::new(RepositoryErrorKind::InvalidData, message)
}

fn repository_error(
    kind: RepositoryErrorKind,
    context: &str,
    error: impl std::fmt::Display,
) -> RepositoryError {
    RepositoryError::new(kind, format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use tempfile::tempdir;
    use typepulse_core::{
        AppPreferences, CompletedSessionRecord, LocalDate, MetricBucketRecord, StatisticsRepository,
    };

    use super::{SqliteStatisticsRepository, LATEST_SCHEMA_VERSION};

    fn session(date: LocalDate, start: i64, chars: u64) -> CompletedSessionRecord {
        CompletedSessionRecord {
            local_date: date,
            started_at_unix_ms: start,
            ended_at_unix_ms: start + 5_000,
            estimated_character_count: chars,
            estimated_word_count: chars as f64 / 5.0,
            average_wpm: 48.0,
            peak_wpm: 72.0,
            active_typing_duration: Duration::from_secs(5),
        }
    }

    #[test]
    fn new_database_migrates_to_latest_schema() {
        let repository = SqliteStatisticsRepository::open_in_memory().unwrap();
        assert_eq!(repository.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    }

    #[test]
    fn sessions_preferences_and_buckets_survive_reopen() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("typepulse.sqlite3");
        let date = LocalDate::new(2026, 8, 5).unwrap();
        {
            let mut repository = SqliteStatisticsRepository::open(&path).unwrap();
            repository.save_session(session(date, 1_000, 25)).unwrap();
            repository
                .save_bucket(MetricBucketRecord {
                    local_date: date,
                    interval_start_unix_ms: 0,
                    interval_duration: Duration::from_secs(60),
                    estimated_character_count: 25,
                    average_wpm: 48.0,
                    peak_wpm: 72.0,
                })
                .unwrap();
            repository
                .save_preferences(AppPreferences {
                    auto_start_enabled: true,
                })
                .unwrap();
        }
        let mut reopened = SqliteStatisticsRepository::open(&path).unwrap();
        assert_eq!(
            reopened.daily_summary(date).unwrap().estimated_word_count,
            5.0
        );
        assert_eq!(reopened.metric_buckets(date).unwrap().len(), 1);
        assert!(reopened.load_preferences().unwrap().auto_start_enabled);
    }

    #[test]
    fn bucket_upsert_merges_and_reset_is_day_scoped() {
        let first_day = LocalDate::new(2026, 8, 5).unwrap();
        let second_day = first_day.next_day().unwrap();
        let mut repository = SqliteStatisticsRepository::open_in_memory().unwrap();
        repository
            .save_session(session(first_day, 1_000, 10))
            .unwrap();
        repository
            .save_session(session(second_day, 2_000, 10))
            .unwrap();
        for average in [20.0, 40.0] {
            repository
                .save_bucket(MetricBucketRecord {
                    local_date: first_day,
                    interval_start_unix_ms: 0,
                    interval_duration: Duration::from_secs(60),
                    estimated_character_count: 10,
                    average_wpm: average,
                    peak_wpm: average,
                })
                .unwrap();
        }
        let bucket = repository.metric_buckets(first_day).unwrap()[0];
        assert_eq!(bucket.estimated_character_count, 20);
        assert_eq!(bucket.average_wpm, 30.0);
        assert_eq!(bucket.peak_wpm, 40.0);

        repository.reset_day(first_day).unwrap();
        assert_eq!(
            repository.daily_summary(first_day).unwrap().session_count,
            0
        );
        assert_eq!(
            repository.daily_summary(second_day).unwrap().session_count,
            1
        );
    }

    #[test]
    fn pre_migration_backup_is_created_for_existing_database() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("typepulse.sqlite3");
        {
            let connection = rusqlite::Connection::open(&path).unwrap();
            connection
                .execute_batch("CREATE TABLE legacy(value TEXT); INSERT INTO legacy VALUES ('ok');")
                .unwrap();
        }
        let repository = SqliteStatisticsRepository::open(&path).unwrap();
        let backup = repository.last_backup_path().unwrap();
        assert!(backup.exists());
        assert!(fs::metadata(backup).unwrap().len() > 0);
    }

    #[test]
    fn schema_has_no_sensitive_input_columns() {
        let repository = SqliteStatisticsRepository::open_in_memory().unwrap();
        let mut statement = repository
            .connection()
            .prepare(
                "SELECT lower(column_info.name)
                 FROM sqlite_schema AS schema,
                      pragma_table_info(schema.name) AS column_info
                 WHERE schema.type = 'table' AND schema.name NOT LIKE 'sqlite_%'",
            )
            .unwrap();
        let columns = statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();
        for forbidden in [
            "key_code",
            "keycode",
            "raw_key",
            "rawkey",
            "text",
            "content",
            "application_name",
            "app_name",
            "window_title",
        ] {
            assert!(
                !columns.iter().any(|column| column == forbidden),
                "forbidden schema column: {forbidden}"
            );
        }
    }
}
