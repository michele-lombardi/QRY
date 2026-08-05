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
    /// Best value in effect when the session started.
    pub previous_wpm: f64,
    /// First displayed value in this session that exceeded the best.
    pub new_wpm: f64,
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
}
