//! Shared runtime state for monitoring, metrics and aggregate persistence.

use std::{
    sync::{
        atomic::{AtomicI64, AtomicU64, Ordering},
        mpsc::RecvTimeoutError,
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use chrono::{DateTime, Datelike, Local, TimeZone};
use typepulse_core::{
    AnimationBand, AppPreferences, CompletedSessionRecord, CoreConfig, DailySummary, EngineUpdate,
    LocalDate, MetricBucketRecord, SessionPhase, SessionSummary, StatisticsRepository,
    TypingEngine,
};
use typepulse_platform_macos::{
    ActivityReceiver, KeyboardMonitor, MonitorConfig, MonitorError, MonitorMetricsSnapshot,
    MonitorRunState,
};
use typepulse_storage_sqlite::SqliteStatisticsRepository;

const BUCKET_DURATION: Duration = Duration::from_secs(60);
const RELAY_TICK: Duration = Duration::from_millis(250);

/// Process-wide owner of monitoring and local aggregate storage.
pub(crate) struct DiagnosticState {
    active: Mutex<Option<ActiveMonitor>>,
    repository: Arc<Mutex<SqliteStatisticsRepository>>,
    total_activities: Arc<AtomicU64>,
    last_activity_unix_ms: Arc<AtomicI64>,
    live_metrics: Arc<Mutex<LiveMetrics>>,
    last_error: Arc<Mutex<Option<String>>>,
}

impl DiagnosticState {
    pub(crate) fn new(repository: SqliteStatisticsRepository) -> Self {
        Self {
            active: Mutex::new(None),
            repository: Arc::new(Mutex::new(repository)),
            total_activities: Arc::new(AtomicU64::new(0)),
            last_activity_unix_ms: Arc::new(AtomicI64::new(0)),
            live_metrics: Arc::new(Mutex::new(LiveMetrics::default())),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn start(&self) -> Result<(), String> {
        let mut active = self.active.lock().map_err(lock_error)?;
        if active.is_some() {
            return Err("input monitoring is already active".into());
        }

        self.total_activities.store(0, Ordering::Relaxed);
        self.last_activity_unix_ms.store(0, Ordering::Relaxed);
        self.set_last_error(None);
        *self
            .live_metrics
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = LiveMetrics::default();

        let (mut monitor, receiver) =
            KeyboardMonitor::start(MonitorConfig::default()).map_err(|error| error.to_string())?;
        let relay_total = Arc::clone(&self.total_activities);
        let relay_last_activity = Arc::clone(&self.last_activity_unix_ms);
        let relay_repository = Arc::clone(&self.repository);
        let relay_live = Arc::clone(&self.live_metrics);
        let relay_error = Arc::clone(&self.last_error);

        let relay_result = std::thread::Builder::new()
            .name("typepulse-metrics-relay".into())
            .spawn(move || {
                relay_activity(
                    receiver,
                    relay_total,
                    relay_last_activity,
                    relay_repository,
                    relay_live,
                    relay_error,
                );
            });
        let relay = match relay_result {
            Ok(relay) => relay,
            Err(error) => {
                let _ = monitor.stop();
                return Err(format!("failed to start metrics relay: {error}"));
            }
        };

        *active = Some(ActiveMonitor {
            monitor,
            relay: Some(relay),
        });
        Ok(())
    }

    pub(crate) fn start_automatically(&self) {
        if let Err(error) = self.start() {
            self.set_last_error(Some(error));
        }
    }

    pub(crate) fn record_runtime_error(&self, error: impl Into<String>) {
        self.set_last_error(Some(error.into()));
    }

    pub(crate) fn stop(&self) -> Result<(), String> {
        let active = self.active.lock().map_err(lock_error)?.take();
        if let Some(mut active) = active {
            active.stop().map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> RuntimeSnapshot {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (run_state, monitor_metrics) = active.as_ref().map_or(
            (MonitorRunState::Stopped, MonitorMetricsSnapshot::default()),
            |active| (active.monitor.state(), active.monitor.metrics()),
        );
        RuntimeSnapshot {
            run_state,
            monitor_metrics,
            total_activities: self.total_activities.load(Ordering::Relaxed),
            last_activity_unix_ms: self.last_activity_unix_ms.load(Ordering::Relaxed),
            live_metrics: *self
                .live_metrics
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
            last_error: self
                .last_error
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
        }
    }

    pub(crate) fn today_summary(&self) -> Result<DailySummary, String> {
        let today = current_wall_observation().date;
        self.repository
            .lock()
            .map_err(lock_error)?
            .daily_summary(today)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn recent_summaries(&self, days: usize) -> Result<Vec<DailySummary>, String> {
        let today = current_wall_observation().date;
        self.repository
            .lock()
            .map_err(lock_error)?
            .recent_daily_summaries(today, days)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn today_buckets(&self) -> Result<Vec<MetricBucketRecord>, String> {
        let today = current_wall_observation().date;
        self.repository
            .lock()
            .map_err(lock_error)?
            .metric_buckets(today)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn reset_today(&self) -> Result<(), String> {
        let today = current_wall_observation().date;
        self.repository
            .lock()
            .map_err(lock_error)?
            .reset_day(today)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn load_preferences(&self) -> Result<AppPreferences, String> {
        self.repository
            .lock()
            .map_err(lock_error)?
            .load_preferences()
            .map_err(|error| error.to_string())
    }

    pub(crate) fn save_preferences(&self, preferences: AppPreferences) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(lock_error)?
            .save_preferences(preferences)
            .map_err(|error| error.to_string())
    }

    fn set_last_error(&self, error: Option<String>) {
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = error;
    }
}

pub(crate) struct RuntimeSnapshot {
    pub(crate) run_state: MonitorRunState,
    pub(crate) monitor_metrics: MonitorMetricsSnapshot,
    pub(crate) total_activities: u64,
    pub(crate) last_activity_unix_ms: i64,
    pub(crate) live_metrics: LiveMetrics,
    pub(crate) last_error: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LiveMetrics {
    pub(crate) phase: SessionPhase,
    pub(crate) raw_wpm: f64,
    pub(crate) displayed_wpm: f64,
    pub(crate) animation_band: AnimationBand,
    pub(crate) active_typing_seconds: f64,
    pub(crate) current_session_characters: u64,
    pub(crate) current_session_average_wpm: f64,
    pub(crate) current_session_peak_wpm: f64,
    pub(crate) personal_best_wpm: f64,
    pub(crate) celebration_sequence: u64,
}

impl Default for LiveMetrics {
    fn default() -> Self {
        Self {
            phase: SessionPhase::Idle,
            raw_wpm: 0.0,
            displayed_wpm: 0.0,
            animation_band: AnimationBand::Still,
            active_typing_seconds: 0.0,
            current_session_characters: 0,
            current_session_average_wpm: 0.0,
            current_session_peak_wpm: 0.0,
            personal_best_wpm: 0.0,
            celebration_sequence: 0,
        }
    }
}

struct ActiveMonitor {
    monitor: KeyboardMonitor,
    relay: Option<JoinHandle<()>>,
}

impl ActiveMonitor {
    fn stop(&mut self) -> Result<(), MonitorError> {
        self.monitor.stop()?;
        if let Some(relay) = self.relay.take() {
            relay.join().map_err(|_| MonitorError::ThreadPanicked)?;
        }
        Ok(())
    }
}

fn relay_activity(
    receiver: ActivityReceiver,
    total: Arc<AtomicU64>,
    last_activity_unix_ms: Arc<AtomicI64>,
    repository: Arc<Mutex<SqliteStatisticsRepository>>,
    live_metrics: Arc<Mutex<LiveMetrics>>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let personal_best = repository
        .lock()
        .ok()
        .and_then(|repository| repository.personal_best_wpm().ok())
        .flatten()
        .filter(|peak| *peak > 0.0);
    let mut engine = match personal_best {
        Some(personal_best) => TypingEngine::with_record(CoreConfig::default(), personal_best),
        None => TypingEngine::new(CoreConfig::default()),
    }
    .expect("default core config and stored non-negative record are valid");
    let mut session_context: Option<SessionWallContext> = None;
    let mut bucket: Option<BucketAccumulator> = None;

    loop {
        match receiver.recv_timeout(RELAY_TICK) {
            Ok(activity) => {
                let wall = current_wall_observation();
                last_activity_unix_ms.store(wall.unix_ms, Ordering::Relaxed);
                if session_context.is_some_and(|context| context.date != wall.date) {
                    if let Some(summary) = engine.finish_active_session() {
                        persist_summary(&repository, session_context.take(), summary, &last_error);
                    }
                }
                rotate_bucket_if_needed(&repository, &mut bucket, wall, &last_error);

                match engine.record_activity(activity) {
                    Ok(update) => {
                        if let Some(summary) = update.completed_session {
                            persist_summary(
                                &repository,
                                session_context.take(),
                                summary,
                                &last_error,
                            );
                        }
                        if session_context.is_none() {
                            session_context = Some(SessionWallContext {
                                date: wall.date,
                                started_at_unix_ms: wall.unix_ms,
                            });
                        }
                        bucket
                            .get_or_insert_with(|| BucketAccumulator::new(wall))
                            .record(update.snapshot.displayed_wpm);
                        total.fetch_add(1, Ordering::Relaxed);
                        set_live_metrics(&live_metrics, update);
                    }
                    Err(error) => set_shared_error(&last_error, error.to_string()),
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                let wall = current_wall_observation();
                if session_context.is_some_and(|context| context.date != wall.date) {
                    if let Some(summary) = engine.finish_active_session() {
                        persist_summary(&repository, session_context.take(), summary, &last_error);
                    }
                }
                rotate_bucket_if_needed(&repository, &mut bucket, wall, &last_error);
                match engine.tick() {
                    Ok(update) => {
                        if let Some(summary) = update.completed_session {
                            persist_summary(
                                &repository,
                                session_context.take(),
                                summary,
                                &last_error,
                            );
                        }
                        set_live_metrics(&live_metrics, update);
                    }
                    Err(error) => set_shared_error(&last_error, error.to_string()),
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    if let Some(summary) = engine.finish_active_session() {
        persist_summary(&repository, session_context, summary, &last_error);
    }
    flush_bucket(&repository, bucket, &last_error);
    *live_metrics
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = LiveMetrics::default();
}

fn persist_summary(
    repository: &Arc<Mutex<SqliteStatisticsRepository>>,
    context: Option<SessionWallContext>,
    summary: SessionSummary,
    last_error: &Arc<Mutex<Option<String>>>,
) {
    let Some(context) = context else {
        set_shared_error(last_error, "missing wall-clock session context".into());
        return;
    };
    let elapsed_ms = duration_millis_i64(summary.elapsed_duration);
    let record = CompletedSessionRecord {
        local_date: context.date,
        started_at_unix_ms: context.started_at_unix_ms,
        ended_at_unix_ms: context.started_at_unix_ms.saturating_add(elapsed_ms),
        estimated_character_count: summary.estimated_character_count,
        estimated_word_count: summary.estimated_word_count,
        average_wpm: summary.average_wpm,
        peak_wpm: summary.peak_wpm,
        active_typing_duration: summary.active_typing_duration,
    };
    let result = repository
        .lock()
        .map_err(lock_error)
        .and_then(|mut repository| {
            repository
                .save_session(record)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        set_shared_error(last_error, error);
    }
}

fn rotate_bucket_if_needed(
    repository: &Arc<Mutex<SqliteStatisticsRepository>>,
    bucket: &mut Option<BucketAccumulator>,
    wall: WallObservation,
    last_error: &Arc<Mutex<Option<String>>>,
) {
    let new_start = bucket_start(wall.unix_ms);
    if bucket.as_ref().is_some_and(|current| {
        current.interval_start_unix_ms != new_start || current.date != wall.date
    }) {
        flush_bucket(repository, bucket.take(), last_error);
    }
}

fn flush_bucket(
    repository: &Arc<Mutex<SqliteStatisticsRepository>>,
    bucket: Option<BucketAccumulator>,
    last_error: &Arc<Mutex<Option<String>>>,
) {
    let Some(bucket) = bucket.filter(|bucket| bucket.character_count > 0) else {
        return;
    };
    let record = bucket.record_value();
    let result = repository
        .lock()
        .map_err(lock_error)
        .and_then(|mut repository| {
            repository
                .save_bucket(record)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        set_shared_error(last_error, error);
    }
}

fn set_live_metrics(target: &Arc<Mutex<LiveMetrics>>, update: EngineUpdate) {
    let mut metrics = target.lock().unwrap_or_else(|error| error.into_inner());
    if update.new_record.is_some() {
        metrics.celebration_sequence = metrics.celebration_sequence.saturating_add(1);
    }
    metrics.phase = update.snapshot.phase;
    metrics.raw_wpm = update.snapshot.raw_wpm;
    metrics.displayed_wpm = update.snapshot.displayed_wpm;
    metrics.animation_band = update.snapshot.animation_band;
    metrics.active_typing_seconds = update
        .snapshot
        .active_session
        .map_or(0.0, |session| session.active_typing_duration.as_secs_f64());
    metrics.current_session_characters = update
        .snapshot
        .active_session
        .map_or(0, |session| session.estimated_character_count);
    metrics.current_session_average_wpm = update
        .snapshot
        .active_session
        .map_or(0.0, |session| session.average_wpm);
    metrics.current_session_peak_wpm = update
        .snapshot
        .active_session
        .map_or(0.0, |session| session.peak_wpm);
    metrics.personal_best_wpm = update.snapshot.personal_best_wpm.unwrap_or(0.0);
}

fn set_shared_error(target: &Arc<Mutex<Option<String>>>, error: String) {
    *target
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(error);
}

#[derive(Clone, Copy)]
struct SessionWallContext {
    date: LocalDate,
    started_at_unix_ms: i64,
}

#[derive(Clone, Copy)]
struct WallObservation {
    date: LocalDate,
    unix_ms: i64,
}

fn current_wall_observation() -> WallObservation {
    let now = Local::now();
    WallObservation {
        date: local_date(&now),
        unix_ms: now.timestamp_millis(),
    }
}

fn local_date<Tz: TimeZone>(value: &DateTime<Tz>) -> LocalDate {
    LocalDate::new(value.year(), value.month() as u8, value.day() as u8)
        .expect("chrono returned a valid local date")
}

struct BucketAccumulator {
    date: LocalDate,
    interval_start_unix_ms: i64,
    character_count: u64,
    displayed_wpm_total: f64,
    peak_wpm: f64,
}

impl BucketAccumulator {
    fn new(wall: WallObservation) -> Self {
        Self {
            date: wall.date,
            interval_start_unix_ms: bucket_start(wall.unix_ms),
            character_count: 0,
            displayed_wpm_total: 0.0,
            peak_wpm: 0.0,
        }
    }

    fn record(&mut self, displayed_wpm: f64) {
        self.character_count = self.character_count.saturating_add(1);
        self.displayed_wpm_total += displayed_wpm;
        self.peak_wpm = self.peak_wpm.max(displayed_wpm);
    }

    fn record_value(&self) -> MetricBucketRecord {
        MetricBucketRecord {
            local_date: self.date,
            interval_start_unix_ms: self.interval_start_unix_ms,
            interval_duration: BUCKET_DURATION,
            estimated_character_count: self.character_count,
            average_wpm: self.displayed_wpm_total / self.character_count as f64,
            peak_wpm: self.peak_wpm,
        }
    }
}

const fn bucket_start(unix_ms: i64) -> i64 {
    unix_ms - unix_ms.rem_euclid(60_000)
}

fn duration_millis_i64(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn lock_error<T>(_error: std::sync::PoisonError<T>) -> String {
    "shared QRY state is unavailable".into()
}

#[cfg(test)]
mod tests {
    use super::{bucket_start, BucketAccumulator, WallObservation, BUCKET_DURATION};
    use chrono::{FixedOffset, TimeZone, Utc};
    use typepulse_core::LocalDate;

    #[test]
    fn buckets_align_to_absolute_minutes_and_aggregate() {
        let wall = WallObservation {
            date: LocalDate::new(2026, 8, 5).unwrap(),
            unix_ms: 125_678,
        };
        assert_eq!(bucket_start(wall.unix_ms), 120_000);
        let mut bucket = BucketAccumulator::new(wall);
        bucket.record(20.0);
        bucket.record(40.0);
        let record = bucket.record_value();
        assert_eq!(record.interval_duration, BUCKET_DURATION);
        assert_eq!(record.estimated_character_count, 2);
        assert_eq!(record.average_wpm, 30.0);
        assert_eq!(record.peak_wpm, 40.0);
    }

    #[test]
    fn one_instant_can_belong_to_distinct_local_dates_without_mixing_them() {
        let instant = Utc.with_ymd_and_hms(2026, 8, 5, 23, 30, 0).unwrap();
        let west = instant.with_timezone(&FixedOffset::west_opt(7 * 3_600).unwrap());
        let east = instant.with_timezone(&FixedOffset::east_opt(9 * 3_600).unwrap());

        assert_eq!(super::local_date(&west).to_string(), "2026-08-05");
        assert_eq!(super::local_date(&east).to_string(), "2026-08-06");
    }
}
