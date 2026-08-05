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

/// Screen corner used by the non-interactive overlay.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayPosition {
    /// Upper-left corner of the selected display work area.
    TopLeft,
    /// Upper-right corner of the selected display work area.
    #[default]
    TopRight,
    /// Lower-left corner of the selected display work area.
    BottomLeft,
    /// Lower-right corner of the selected display work area.
    BottomRight,
}

impl OverlayPosition {
    /// Stable storage and DTO representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
        }
    }

    /// Parses the stable storage representation.
    #[must_use]
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "top-left" => Some(Self::TopLeft),
            "top-right" => Some(Self::TopRight),
            "bottom-left" => Some(Self::BottomLeft),
            "bottom-right" => Some(Self::BottomRight),
            _ => None,
        }
    }
}

/// Preset dimensions for the overlay card.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlaySize {
    /// Compact card.
    Small,
    /// Default card.
    #[default]
    Medium,
    /// Large card for improved readability.
    Large,
}

impl OverlaySize {
    /// Stable storage and DTO representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    /// Parses the stable storage representation.
    #[must_use]
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            _ => None,
        }
    }
}

/// Information rendered inside the overlay.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayContent {
    /// Numeric live WPM only.
    Wpm,
    /// Animated character only.
    Animation,
    /// Animated character and live WPM.
    #[default]
    Both,
}

impl OverlayContent {
    /// Stable storage and DTO representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wpm => "wpm",
            Self::Animation => "animation",
            Self::Both => "both",
        }
    }

    /// Parses the stable storage representation.
    #[must_use]
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "wpm" => Some(Self::Wpm),
            "animation" => Some(Self::Animation),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// Locally persisted application preferences.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppPreferences {
    /// Launch at OS login and automatically start monitoring when the app opens.
    pub auto_start_enabled: bool,
    /// Whether live typing may present the overlay.
    pub overlay_enabled: bool,
    /// Corner of the active primary display work area.
    pub overlay_position: OverlayPosition,
    /// Overlay card dimension preset.
    pub overlay_size: OverlaySize,
    /// Overlay visual content.
    pub overlay_content: OverlayContent,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            auto_start_enabled: false,
            overlay_enabled: true,
            overlay_position: OverlayPosition::TopRight,
            overlay_size: OverlaySize::Medium,
            overlay_content: OverlayContent::Both,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppPreferences, OverlayContent, OverlayPosition, OverlaySize};

    #[test]
    fn overlay_defaults_and_storage_values_are_stable() {
        let defaults = AppPreferences::default();
        assert!(defaults.overlay_enabled);
        assert_eq!(defaults.overlay_position, OverlayPosition::TopRight);
        assert_eq!(defaults.overlay_size, OverlaySize::Medium);
        assert_eq!(defaults.overlay_content, OverlayContent::Both);

        for value in [
            OverlayPosition::TopLeft,
            OverlayPosition::TopRight,
            OverlayPosition::BottomLeft,
            OverlayPosition::BottomRight,
        ] {
            assert_eq!(OverlayPosition::from_stored(value.as_str()), Some(value));
        }
        for value in [OverlaySize::Small, OverlaySize::Medium, OverlaySize::Large] {
            assert_eq!(OverlaySize::from_stored(value.as_str()), Some(value));
        }
        for value in [
            OverlayContent::Wpm,
            OverlayContent::Animation,
            OverlayContent::Both,
        ] {
            assert_eq!(OverlayContent::from_stored(value.as_str()), Some(value));
        }
    }
}
