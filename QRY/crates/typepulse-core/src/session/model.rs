//! Public, platform-independent output produced by the typing engine.

use std::time::{Duration, Instant};

use crate::AnimationBand;

/// High-level session and overlay state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionPhase {
    /// No session exists.
    Idle,
    /// A session exists and the overlay should be visible.
    ActiveVisible,
    /// A session exists but the overlay inactivity delay elapsed.
    ActiveHidden,
}

impl SessionPhase {
    /// Stable string for application DTO conversion.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::ActiveVisible => "active-visible",
            Self::ActiveHidden => "active-hidden",
        }
    }

    /// Whether the overlay should currently be presented.
    #[must_use]
    pub const fn overlay_visible(self) -> bool {
        matches!(self, Self::ActiveVisible)
    }
}

/// Running aggregates for the active session.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveSessionMetrics {
    /// Monotonic start of the session.
    pub started_at: Instant,
    /// Most recent typing activity.
    pub last_activity_at: Instant,
    /// Count of privacy-safe typing activities.
    pub estimated_character_count: u64,
    /// Character count divided by five.
    pub estimated_word_count: f64,
    /// Arithmetic mean of displayed WPM samples after record qualification.
    pub average_wpm: f64,
    /// Highest qualified displayed WPM reached by the session.
    pub peak_wpm: f64,
    /// Sum of inter-activity gaps no longer than the configured active limit.
    pub active_typing_duration: Duration,
}

/// Immutable aggregate emitted once a session ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SessionSummary {
    /// Monotonic first activity.
    pub started_at: Instant,
    /// Monotonic last activity; the trailing idle timeout is excluded.
    pub ended_at: Instant,
    /// Elapsed time from first through last activity.
    pub elapsed_duration: Duration,
    /// Total privacy-safe activity count.
    pub estimated_character_count: u64,
    /// Character count divided by five.
    pub estimated_word_count: f64,
    /// Mean displayed WPM sampled after record qualification.
    pub average_wpm: f64,
    /// Highest qualified displayed WPM reached by the session.
    pub peak_wpm: f64,
    /// Sum of short inter-activity gaps, excluding idle pauses.
    pub active_typing_duration: Duration,
}

/// One-time notification that an existing personal best was exceeded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewRecord {
    /// Record category that was exceeded.
    pub kind: RecordKind,
    /// Best value in effect when the session started.
    pub previous_wpm: f64,
    /// First displayed value in this session that exceeded the best.
    pub new_wpm: f64,
}

/// Personal record category used by the shared celebration event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// Highest qualified short rolling peak.
    Peak,
    /// Best pace sustained across a complete 30 second window.
    Sustained30Seconds,
    /// Best pace sustained across a complete 60 second window.
    Sustained60Seconds,
}

/// Aggregate all-time WPM records persisted locally.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TypingRecords {
    /// Highest qualified short rolling peak.
    pub peak_wpm: Option<f64>,
    /// Best fixed 30 second window.
    pub sustained_30_wpm: Option<f64>,
    /// Best fixed 60 second window.
    pub sustained_60_wpm: Option<f64>,
}

impl TypingRecords {
    /// Whether every stored value is finite and non-negative.
    #[must_use]
    pub fn is_valid(self) -> bool {
        [self.peak_wpm, self.sustained_30_wpm, self.sustained_60_wpm]
            .into_iter()
            .flatten()
            .all(|value| value.is_finite() && value >= 0.0)
    }
}

/// Complete state needed by a future tray or overlay adapter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineSnapshot {
    /// Monotonic instant represented by this snapshot.
    pub observed_at: Instant,
    /// Session/overlay lifecycle state.
    pub phase: SessionPhase,
    /// Unsmoothed rolling WPM.
    pub raw_wpm: f64,
    /// Smoothed value intended for display and animation.
    pub displayed_wpm: f64,
    /// Animation intensity selected from displayed WPM.
    pub animation_band: AnimationBand,
    /// Running session aggregates, absent while idle.
    pub active_session: Option<ActiveSessionMetrics>,
    /// Best WPM observed by this engine, when established.
    pub personal_best_wpm: Option<f64>,
    /// Best pace sustained for 30 seconds.
    pub sustained_30_best_wpm: Option<f64>,
    /// Best pace sustained for 60 seconds.
    pub sustained_60_best_wpm: Option<f64>,
}

/// Output from processing an activity or advancing the engine clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineUpdate {
    /// Current state after the transition.
    pub snapshot: EngineSnapshot,
    /// Session completed by this transition, emitted exactly once.
    pub completed_session: Option<SessionSummary>,
    /// Record celebration emitted at most once in a session.
    pub new_record: Option<NewRecord>,
    /// Whether one or more persisted record values changed.
    pub records_updated: bool,
}
