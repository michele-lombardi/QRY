//! Persistible aggregate models with no individual input events.

use std::time::Duration;

use crate::LocalDate;

/// Aggregate representation of one completed session.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompletedSessionRecord {
    /// Local calendar day assigned by the application at session start.
    pub local_date: LocalDate,
    /// Session start as Unix milliseconds.
    pub started_at_unix_ms: i64,
    /// Last activity as Unix milliseconds.
    pub ended_at_unix_ms: i64,
    /// Aggregate character estimate.
    pub estimated_character_count: u64,
    /// Aggregate word estimate.
    pub estimated_word_count: f64,
    /// Mean displayed WPM sampled on activity.
    pub average_wpm: f64,
    /// Maximum displayed WPM.
    pub peak_wpm: f64,
    /// Active typing time excluding long idle gaps.
    pub active_typing_duration: Duration,
}

/// Aggregate sample for a fixed chart interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricBucketRecord {
    /// Local day containing the interval start.
    pub local_date: LocalDate,
    /// Interval start as Unix milliseconds.
    pub interval_start_unix_ms: i64,
    /// Fixed bucket duration.
    pub interval_duration: Duration,
    /// Aggregate character estimate in this interval.
    pub estimated_character_count: u64,
    /// Mean displayed WPM sampled on activity in this interval.
    pub average_wpm: f64,
    /// Maximum displayed WPM in this interval.
    pub peak_wpm: f64,
}

/// Complete summary for one local calendar day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DailySummary {
    /// Day represented by the summary.
    pub date: LocalDate,
    /// Aggregate activity count from completed sessions.
    pub estimated_character_count: u64,
    /// Aggregate words from completed sessions.
    pub estimated_word_count: f64,
    /// Character-weighted mean session WPM.
    pub average_wpm: f64,
    /// Highest session WPM.
    pub peak_wpm: f64,
    /// Sum of active typing durations.
    pub active_typing_duration: Duration,
    /// Number of completed sessions.
    pub session_count: u64,
}

impl DailySummary {
    /// Creates an empty summary used for days with no stored activity.
    #[must_use]
    pub const fn empty(date: LocalDate) -> Self {
        Self {
            date,
            estimated_character_count: 0,
            estimated_word_count: 0.0,
            average_wpm: 0.0,
            peak_wpm: 0.0,
            active_typing_duration: Duration::ZERO,
            session_count: 0,
        }
    }
}

/// Locally persisted application preferences introduced with Phase D.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AppPreferences {
    /// Launch at OS login and automatically start monitoring when the app opens.
    pub auto_start_enabled: bool,
}
