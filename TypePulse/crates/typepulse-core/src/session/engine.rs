//! Deterministic state machine combining live metrics and session aggregates.

use std::{
    fmt,
    time::{Duration, Instant},
};

use crate::{
    metrics::{ExponentialSmoother, RollingWpm},
    ActiveSessionMetrics, Clock, ConfigError, CoreConfig, EngineSnapshot, EngineUpdate, NewRecord,
    SessionPhase, SessionSummary, SystemClock, TypingActivity,
};

/// Portable typing engine parameterized by its monotonic clock.
#[derive(Clone, Debug)]
pub struct TypingEngine<C: Clock = SystemClock> {
    config: CoreConfig,
    clock: C,
    rolling_wpm: RollingWpm,
    smoother: ExponentialSmoother,
    active_session: Option<ActiveSession>,
    personal_best_wpm: Option<f64>,
}

impl TypingEngine<SystemClock> {
    /// Creates a production engine using the system monotonic clock.
    pub fn new(config: CoreConfig) -> Result<Self, EngineError> {
        Self::with_clock_and_record(config, SystemClock, None)
    }

    /// Creates a production engine with an existing personal record.
    pub fn with_record(config: CoreConfig, personal_best_wpm: f64) -> Result<Self, EngineError> {
        Self::with_clock_and_record(config, SystemClock, Some(personal_best_wpm))
    }
}

impl<C: Clock> TypingEngine<C> {
    /// Creates an engine with an injected clock and no historical record.
    pub fn with_clock(config: CoreConfig, clock: C) -> Result<Self, EngineError> {
        Self::with_clock_and_record(config, clock, None)
    }

    /// Creates an engine with an injected clock and optional historical record.
    pub fn with_clock_and_record(
        config: CoreConfig,
        clock: C,
        personal_best_wpm: Option<f64>,
    ) -> Result<Self, EngineError> {
        let config = config.validate().map_err(EngineError::InvalidConfig)?;
        if personal_best_wpm.is_some_and(|value| !value.is_finite() || value < 0.0) {
            return Err(EngineError::InvalidPersonalBest);
        }

        Ok(Self {
            config,
            clock,
            rolling_wpm: RollingWpm::new(config.rolling_window),
            smoother: ExponentialSmoother::new(config.smoothing_factor),
            active_session: None,
            personal_best_wpm,
        })
    }

    /// Records one activity at the current injected-clock instant.
    pub fn record_now(&mut self) -> Result<EngineUpdate, EngineError> {
        self.record_activity(TypingActivity::at(self.clock.now()))
    }

    /// Records an externally timestamped privacy-safe activity.
    pub fn record_activity(
        &mut self,
        activity: TypingActivity,
    ) -> Result<EngineUpdate, EngineError> {
        let occurred_at = activity.occurred_at();
        self.ensure_not_before_last_activity(occurred_at)?;

        let completed_session = self.complete_if_timed_out(occurred_at);
        if self.active_session.is_none() {
            self.rolling_wpm.reset();
            self.smoother.reset();
            self.active_session = Some(ActiveSession::new(occurred_at, self.personal_best_wpm));
        }

        let raw_wpm = self.rolling_wpm.record(occurred_at);
        let displayed_wpm = self.smoother.update(raw_wpm);
        let active_session = self.active_session.as_mut().expect("session was created");
        active_session.record(occurred_at, displayed_wpm, self.config.active_gap_limit);

        let new_record = active_session.take_record(displayed_wpm);
        self.personal_best_wpm = Some(
            self.personal_best_wpm
                .map_or(displayed_wpm, |best| best.max(displayed_wpm)),
        );

        Ok(EngineUpdate {
            snapshot: self.snapshot_at(occurred_at),
            completed_session,
            new_record,
        })
    }

    /// Advances lifecycle timers using the injected clock.
    pub fn tick(&mut self) -> Result<EngineUpdate, EngineError> {
        self.tick_at(self.clock.now())
    }

    /// Advances lifecycle timers to an explicit instant.
    pub fn tick_at(&mut self, now: Instant) -> Result<EngineUpdate, EngineError> {
        self.ensure_not_before_last_activity(now)?;
        let completed_session = self.complete_if_timed_out(now);
        Ok(EngineUpdate {
            snapshot: self.snapshot_at(now),
            completed_session,
            new_record: None,
        })
    }

    /// Returns the validated parameters used by this engine.
    #[must_use]
    pub const fn config(&self) -> CoreConfig {
        self.config
    }

    fn ensure_not_before_last_activity(&self, instant: Instant) -> Result<(), EngineError> {
        if self
            .active_session
            .as_ref()
            .is_some_and(|session| instant < session.last_activity_at)
        {
            Err(EngineError::NonMonotonicTime)
        } else {
            Ok(())
        }
    }

    fn complete_if_timed_out(&mut self, now: Instant) -> Option<SessionSummary> {
        let should_complete = self.active_session.as_ref().is_some_and(|session| {
            elapsed_since(now, session.last_activity_at) >= self.config.session_end_after
        });
        if !should_complete {
            return None;
        }

        let summary = self.active_session.take().map(ActiveSession::summary);
        self.rolling_wpm.reset();
        self.smoother.reset();
        summary
    }

    fn snapshot_at(&mut self, now: Instant) -> EngineSnapshot {
        let raw_wpm = self.rolling_wpm.at(now);
        let displayed_wpm = if self.active_session.is_some() {
            self.smoother.value()
        } else {
            0.0
        };
        let phase = self
            .active_session
            .as_ref()
            .map_or(SessionPhase::Idle, |session| {
                if elapsed_since(now, session.last_activity_at) >= self.config.overlay_hide_after {
                    SessionPhase::ActiveHidden
                } else {
                    SessionPhase::ActiveVisible
                }
            });

        EngineSnapshot {
            observed_at: now,
            phase,
            raw_wpm,
            displayed_wpm,
            animation_band: self.config.animation_thresholds.band_for(displayed_wpm),
            active_session: self.active_session.as_ref().map(ActiveSession::metrics),
            personal_best_wpm: self.personal_best_wpm,
        }
    }
}

#[derive(Clone, Debug)]
struct ActiveSession {
    started_at: Instant,
    last_activity_at: Instant,
    estimated_character_count: u64,
    displayed_wpm_total: f64,
    displayed_wpm_samples: u64,
    peak_wpm: f64,
    active_typing_duration: Duration,
    record_to_beat: Option<f64>,
    record_emitted: bool,
}

impl ActiveSession {
    fn new(started_at: Instant, record_to_beat: Option<f64>) -> Self {
        Self {
            started_at,
            last_activity_at: started_at,
            estimated_character_count: 0,
            displayed_wpm_total: 0.0,
            displayed_wpm_samples: 0,
            peak_wpm: 0.0,
            active_typing_duration: Duration::ZERO,
            record_to_beat,
            record_emitted: false,
        }
    }

    fn record(&mut self, occurred_at: Instant, displayed_wpm: f64, active_gap_limit: Duration) {
        if self.estimated_character_count > 0 {
            let gap = elapsed_since(occurred_at, self.last_activity_at);
            if gap <= active_gap_limit {
                self.active_typing_duration = self.active_typing_duration.saturating_add(gap);
            }
        }
        self.last_activity_at = occurred_at;
        self.estimated_character_count = self.estimated_character_count.saturating_add(1);
        self.displayed_wpm_samples = self.displayed_wpm_samples.saturating_add(1);
        self.displayed_wpm_total = finite_add(self.displayed_wpm_total, displayed_wpm);
        self.peak_wpm = self.peak_wpm.max(displayed_wpm);
    }

    fn take_record(&mut self, displayed_wpm: f64) -> Option<NewRecord> {
        if self.record_emitted {
            return None;
        }
        let previous_wpm = self.record_to_beat?;
        if displayed_wpm <= previous_wpm {
            return None;
        }
        self.record_emitted = true;
        Some(NewRecord {
            previous_wpm,
            new_wpm: displayed_wpm,
        })
    }

    fn average_wpm(&self) -> f64 {
        if self.displayed_wpm_samples == 0 {
            0.0
        } else {
            (self.displayed_wpm_total / self.displayed_wpm_samples as f64).max(0.0)
        }
    }

    fn metrics(&self) -> ActiveSessionMetrics {
        ActiveSessionMetrics {
            started_at: self.started_at,
            last_activity_at: self.last_activity_at,
            estimated_character_count: self.estimated_character_count,
            estimated_word_count: self.estimated_character_count as f64 / 5.0,
            average_wpm: self.average_wpm(),
            peak_wpm: self.peak_wpm,
            active_typing_duration: self.active_typing_duration,
        }
    }

    fn summary(self) -> SessionSummary {
        SessionSummary {
            started_at: self.started_at,
            ended_at: self.last_activity_at,
            elapsed_duration: elapsed_since(self.last_activity_at, self.started_at),
            estimated_character_count: self.estimated_character_count,
            estimated_word_count: self.estimated_character_count as f64 / 5.0,
            average_wpm: self.average_wpm(),
            peak_wpm: self.peak_wpm,
            active_typing_duration: self.active_typing_duration,
        }
    }
}

fn elapsed_since(later: Instant, earlier: Instant) -> Duration {
    later
        .checked_duration_since(earlier)
        .unwrap_or(Duration::ZERO)
}

fn finite_add(left: f64, right: f64) -> f64 {
    let result = left + right;
    if result.is_finite() {
        result.max(0.0)
    } else {
        f64::MAX
    }
}

/// Invalid input or configuration supplied to the typing engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EngineError {
    /// Core configuration failed validation.
    InvalidConfig(ConfigError),
    /// A supplied activity or tick moved before the latest activity.
    NonMonotonicTime,
    /// Historical WPM record was negative or non-finite.
    InvalidPersonalBest,
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(error) => write!(formatter, "invalid core configuration: {error}"),
            Self::NonMonotonicTime => write!(formatter, "engine time moved backwards"),
            Self::InvalidPersonalBest => {
                write!(formatter, "personal best must be finite and non-negative")
            }
        }
    }
}

impl std::error::Error for EngineError {}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{
        AnimationBand, Clock, CoreConfig, EngineError, ManualClock, SessionPhase, TypingActivity,
        TypingEngine,
    };

    fn engine() -> (ManualClock, TypingEngine<ManualClock>) {
        let clock = ManualClock::new(Instant::now());
        let engine = TypingEngine::with_clock(CoreConfig::default(), clock.clone()).unwrap();
        (clock, engine)
    }

    #[test]
    fn idle_activity_hidden_and_completed_transitions_are_exact() {
        let (clock, mut engine) = engine();
        assert_eq!(engine.tick().unwrap().snapshot.phase, SessionPhase::Idle);

        let first = engine.record_now().unwrap();
        assert_eq!(first.snapshot.phase, SessionPhase::ActiveVisible);
        clock.advance(Duration::from_secs(2)).unwrap();
        assert_eq!(
            engine.tick().unwrap().snapshot.phase,
            SessionPhase::ActiveHidden
        );
        clock.advance(Duration::from_secs(28)).unwrap();
        let ended = engine.tick().unwrap();
        assert_eq!(ended.snapshot.phase, SessionPhase::Idle);
        assert_eq!(
            ended.completed_session.unwrap().estimated_character_count,
            1
        );
        assert!(engine.tick().unwrap().completed_session.is_none());
    }

    #[test]
    fn activity_before_timeout_keeps_the_session_alive() {
        let (clock, mut engine) = engine();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(29)).unwrap();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(29)).unwrap();
        assert_eq!(
            engine.tick().unwrap().snapshot.phase,
            SessionPhase::ActiveHidden
        );
    }

    #[test]
    fn activity_after_timeout_completes_old_session_and_starts_another() {
        let (clock, mut engine) = engine();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(30)).unwrap();
        let update = engine.record_now().unwrap();
        assert_eq!(
            update.completed_session.unwrap().estimated_character_count,
            1
        );
        assert_eq!(
            update
                .snapshot
                .active_session
                .unwrap()
                .estimated_character_count,
            1
        );
    }

    #[test]
    fn aggregates_exclude_long_idle_gaps_from_active_time() {
        let (clock, mut engine) = engine();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(1)).unwrap();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(3)).unwrap();
        engine.record_now().unwrap();
        clock.advance(Duration::from_secs(30)).unwrap();
        let summary = engine.tick().unwrap().completed_session.unwrap();

        assert_eq!(summary.estimated_character_count, 3);
        assert_eq!(summary.estimated_word_count, 0.6);
        assert_eq!(summary.active_typing_duration, Duration::from_secs(1));
        assert_eq!(summary.elapsed_duration, Duration::from_secs(4));
        assert!(summary.average_wpm.is_finite());
        assert!(summary.peak_wpm >= summary.average_wpm);
    }

    #[test]
    fn record_is_emitted_only_once_per_session() {
        let clock = ManualClock::new(Instant::now());
        let mut engine = TypingEngine::with_clock_and_record(
            CoreConfig {
                smoothing_factor: 1.0,
                ..CoreConfig::default()
            },
            clock.clone(),
            Some(2.0),
        )
        .unwrap();

        assert!(engine.record_now().unwrap().new_record.is_none());
        clock.advance(Duration::from_millis(20)).unwrap();
        let record = engine.record_now().unwrap().new_record.unwrap();
        assert_eq!(record.previous_wpm, 2.0);
        assert!((record.new_wpm - 2.4).abs() < 1e-9);
        clock.advance(Duration::from_millis(20)).unwrap();
        assert!(engine.record_now().unwrap().new_record.is_none());
    }

    #[test]
    fn no_historical_record_means_no_first_session_celebration() {
        let (clock, mut engine) = engine();
        for _ in 0..100 {
            assert!(engine.record_now().unwrap().new_record.is_none());
            clock.advance(Duration::from_millis(10)).unwrap();
        }
    }

    #[test]
    fn output_contains_live_wpm_and_animation_band() {
        let (clock, mut engine) = engine();
        let mut update = engine.record_now().unwrap();
        for _ in 1..50 {
            clock.advance(Duration::from_millis(100)).unwrap();
            update = engine.record_now().unwrap();
        }
        assert_eq!(update.snapshot.raw_wpm, 60.0);
        assert_eq!(update.snapshot.animation_band, AnimationBand::Steady);
        assert!(update.snapshot.displayed_wpm > 50.0);
    }

    #[test]
    fn rejects_time_travel_and_invalid_record() {
        let origin = Instant::now();
        let clock = ManualClock::new(origin);
        let mut engine = TypingEngine::with_clock(CoreConfig::default(), clock).unwrap();
        engine
            .record_activity(TypingActivity::at(origin + Duration::from_secs(1)))
            .unwrap();
        assert_eq!(
            engine.record_activity(TypingActivity::at(origin)),
            Err(EngineError::NonMonotonicTime)
        );
        assert!(matches!(
            TypingEngine::with_clock_and_record(
                CoreConfig::default(),
                ManualClock::new(origin),
                Some(f64::NAN)
            ),
            Err(EngineError::InvalidPersonalBest)
        ));
    }

    #[test]
    fn extensive_deterministic_sequences_preserve_numeric_invariants() {
        let (clock, mut engine) = engine();
        let mut state = 0x9E37_79B9_u64;
        for _ in 0..10_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let gap_ms = 1 + state % 2_500;
            clock.advance(Duration::from_millis(gap_ms)).unwrap();
            let update = engine.record_now().unwrap();
            for value in [
                update.snapshot.raw_wpm,
                update.snapshot.displayed_wpm,
                update.snapshot.personal_best_wpm.unwrap_or(0.0),
            ] {
                assert!(value.is_finite() && value >= 0.0);
            }
            let metrics = update.snapshot.active_session.unwrap();
            assert!(metrics.average_wpm.is_finite() && metrics.average_wpm >= 0.0);
            assert!(metrics.peak_wpm >= metrics.average_wpm);
            assert!(
                metrics.active_typing_duration <= clock.now().duration_since(metrics.started_at)
            );
        }
    }
}
