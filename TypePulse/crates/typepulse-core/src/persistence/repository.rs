//! Repository contract and deterministic in-memory implementation.

use std::{collections::BTreeMap, fmt};

use crate::{AppPreferences, CompletedSessionRecord, DailySummary, LocalDate, MetricBucketRecord};

/// Aggregate-only storage operations required by the application.
pub trait StatisticsRepository {
    /// Stores a completed session idempotently.
    fn save_session(&mut self, session: CompletedSessionRecord) -> Result<(), RepositoryError>;
    /// Stores or merges a fixed-duration metric bucket.
    fn save_bucket(&mut self, bucket: MetricBucketRecord) -> Result<(), RepositoryError>;
    /// Returns a summary, including an empty value when the day has no data.
    fn daily_summary(&mut self, date: LocalDate) -> Result<DailySummary, RepositoryError>;
    /// Returns exactly `day_count` chronological summaries ending on `through`.
    fn recent_daily_summaries(
        &mut self,
        through: LocalDate,
        day_count: usize,
    ) -> Result<Vec<DailySummary>, RepositoryError>;
    /// Returns chart buckets for one day ordered by interval start.
    fn metric_buckets(
        &mut self,
        date: LocalDate,
    ) -> Result<Vec<MetricBucketRecord>, RepositoryError>;
    /// Deletes sessions and buckets for exactly one day.
    fn reset_day(&mut self, date: LocalDate) -> Result<(), RepositoryError>;
    /// Loads application preferences, returning defaults for a new repository.
    fn load_preferences(&mut self) -> Result<AppPreferences, RepositoryError>;
    /// Atomically replaces application preferences.
    fn save_preferences(&mut self, preferences: AppPreferences) -> Result<(), RepositoryError>;
}

/// In-memory repository for domain and application tests.
#[derive(Debug, Default)]
pub struct InMemoryStatisticsRepository {
    sessions: Vec<CompletedSessionRecord>,
    buckets: BTreeMap<i64, MetricBucketRecord>,
    preferences: AppPreferences,
}

impl StatisticsRepository for InMemoryStatisticsRepository {
    fn save_session(&mut self, session: CompletedSessionRecord) -> Result<(), RepositoryError> {
        if !self.sessions.iter().any(|existing| {
            existing.started_at_unix_ms == session.started_at_unix_ms
                && existing.ended_at_unix_ms == session.ended_at_unix_ms
        }) {
            self.sessions.push(session);
        }
        Ok(())
    }

    fn save_bucket(&mut self, bucket: MetricBucketRecord) -> Result<(), RepositoryError> {
        self.buckets
            .entry(bucket.interval_start_unix_ms)
            .and_modify(|current| merge_bucket(current, bucket))
            .or_insert(bucket);
        Ok(())
    }

    fn daily_summary(&mut self, date: LocalDate) -> Result<DailySummary, RepositoryError> {
        Ok(summarize_sessions(
            date,
            self.sessions
                .iter()
                .filter(|session| session.local_date == date),
        ))
    }

    fn recent_daily_summaries(
        &mut self,
        through: LocalDate,
        day_count: usize,
    ) -> Result<Vec<DailySummary>, RepositoryError> {
        recent_summaries(through, day_count, |date| self.daily_summary(date))
    }

    fn reset_day(&mut self, date: LocalDate) -> Result<(), RepositoryError> {
        self.sessions.retain(|session| session.local_date != date);
        self.buckets.retain(|_, bucket| bucket.local_date != date);
        Ok(())
    }

    fn metric_buckets(
        &mut self,
        date: LocalDate,
    ) -> Result<Vec<MetricBucketRecord>, RepositoryError> {
        Ok(self
            .buckets
            .values()
            .filter(|bucket| bucket.local_date == date)
            .copied()
            .collect())
    }

    fn load_preferences(&mut self) -> Result<AppPreferences, RepositoryError> {
        Ok(self.preferences)
    }

    fn save_preferences(&mut self, preferences: AppPreferences) -> Result<(), RepositoryError> {
        self.preferences = preferences;
        Ok(())
    }
}

pub(crate) fn summarize_sessions<'a>(
    date: LocalDate,
    sessions: impl Iterator<Item = &'a CompletedSessionRecord>,
) -> DailySummary {
    let mut summary = DailySummary::empty(date);
    let mut weighted_wpm = 0.0;
    for session in sessions {
        summary.estimated_character_count = summary
            .estimated_character_count
            .saturating_add(session.estimated_character_count);
        summary.estimated_word_count =
            finite_add(summary.estimated_word_count, session.estimated_word_count);
        weighted_wpm = finite_add(
            weighted_wpm,
            session.average_wpm * session.estimated_character_count as f64,
        );
        summary.peak_wpm = summary.peak_wpm.max(session.peak_wpm);
        summary.active_typing_duration = summary
            .active_typing_duration
            .saturating_add(session.active_typing_duration);
        summary.session_count = summary.session_count.saturating_add(1);
    }
    if summary.estimated_character_count > 0 {
        summary.average_wpm = weighted_wpm / summary.estimated_character_count as f64;
    }
    summary
}

pub(crate) fn recent_summaries(
    through: LocalDate,
    day_count: usize,
    mut load: impl FnMut(LocalDate) -> Result<DailySummary, RepositoryError>,
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
    dates.into_iter().map(&mut load).collect()
}

fn merge_bucket(current: &mut MetricBucketRecord, incoming: MetricBucketRecord) {
    let old_count = current.estimated_character_count;
    let new_count = incoming.estimated_character_count;
    let total = old_count.saturating_add(new_count);
    if total > 0 {
        current.average_wpm = (current.average_wpm * old_count as f64
            + incoming.average_wpm * new_count as f64)
            / total as f64;
    }
    current.estimated_character_count = total;
    current.peak_wpm = current.peak_wpm.max(incoming.peak_wpm);
}

fn finite_add(left: f64, right: f64) -> f64 {
    let value = left + right;
    if value.is_finite() {
        value.max(0.0)
    } else {
        f64::MAX
    }
}

/// Portable category for storage failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryErrorKind {
    /// Repository could not be opened or initialized.
    Open,
    /// Schema migration did not complete.
    Migration,
    /// Read or write query failed.
    Query,
    /// Stored or supplied aggregate data was invalid.
    InvalidData,
    /// Filesystem operation failed.
    Io,
}

/// Storage error without a dependency on a concrete database library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryError {
    /// Stable failure category.
    pub kind: RepositoryErrorKind,
    /// Aggregate-safe diagnostic message.
    pub message: String,
}

impl RepositoryError {
    /// Creates a categorized repository error.
    #[must_use]
    pub fn new(kind: RepositoryErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for RepositoryError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        AppPreferences, CompletedSessionRecord, InMemoryStatisticsRepository, LocalDate,
        StatisticsRepository,
    };

    fn session(
        date: LocalDate,
        start: i64,
        characters: u64,
        average: f64,
    ) -> CompletedSessionRecord {
        CompletedSessionRecord {
            local_date: date,
            started_at_unix_ms: start,
            ended_at_unix_ms: start + 1_000,
            estimated_character_count: characters,
            estimated_word_count: characters as f64 / 5.0,
            average_wpm: average,
            peak_wpm: average + 10.0,
            active_typing_duration: Duration::from_secs(1),
        }
    }

    #[test]
    fn aggregates_weighted_daily_values_and_is_idempotent() {
        let date = LocalDate::new(2026, 8, 5).unwrap();
        let mut repository = InMemoryStatisticsRepository::default();
        let first = session(date, 1_000, 10, 30.0);
        repository.save_session(first).unwrap();
        repository.save_session(first).unwrap();
        repository
            .save_session(session(date, 2_000, 30, 50.0))
            .unwrap();

        let summary = repository.daily_summary(date).unwrap();
        assert_eq!(summary.estimated_character_count, 40);
        assert_eq!(summary.estimated_word_count, 8.0);
        assert_eq!(summary.average_wpm, 45.0);
        assert_eq!(summary.peak_wpm, 60.0);
        assert_eq!(summary.session_count, 2);
    }

    #[test]
    fn a_new_day_is_empty_without_deleting_history() {
        let previous = LocalDate::new(2026, 8, 5).unwrap();
        let today = previous.next_day().unwrap();
        let mut repository = InMemoryStatisticsRepository::default();
        repository
            .save_session(session(previous, 1_000, 10, 30.0))
            .unwrap();

        assert_eq!(repository.daily_summary(today).unwrap().session_count, 0);
        assert_eq!(repository.daily_summary(previous).unwrap().session_count, 1);
    }

    #[test]
    fn recent_days_include_gaps_and_preferences_round_trip() {
        let through = LocalDate::new(2026, 8, 5).unwrap();
        let mut repository = InMemoryStatisticsRepository::default();
        let summaries = repository.recent_daily_summaries(through, 7).unwrap();
        assert_eq!(summaries.len(), 7);
        assert_eq!(summaries.last().unwrap().date, through);
        assert!(summaries.iter().all(|summary| summary.session_count == 0));

        repository
            .save_preferences(AppPreferences {
                auto_start_enabled: true,
                ..AppPreferences::default()
            })
            .unwrap();
        assert!(repository.load_preferences().unwrap().auto_start_enabled);
    }
}
