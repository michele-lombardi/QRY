//! Platform-independent domain logic for TypePulse.

mod activity;
mod clock;
mod config;
mod date;
mod metrics;
mod persistence;
mod session;

pub use activity::TypingActivity;
pub use clock::{Clock, ClockError, ManualClock, SystemClock};
pub use config::{ConfigError, CoreConfig};
pub use date::{DateError, LocalDate};
pub use metrics::{AnimationBand, AnimationThresholdError, AnimationThresholds};
pub use persistence::{
    export_daily_csv, AppPreferences, CompletedSessionRecord, DailySummary,
    InMemoryStatisticsRepository, MetricBucketRecord, RepositoryError, RepositoryErrorKind,
    StatisticsRepository,
};
pub use session::{
    ActiveSessionMetrics, EngineError, EngineSnapshot, EngineUpdate, NewRecord, SessionPhase,
    SessionSummary, TypingEngine,
};

/// Reports whether the core crate is available to the application workspace.
#[must_use]
pub const fn is_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_crate_is_available() {
        assert!(super::is_ready());
    }
}
