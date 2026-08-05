//! Validated configuration for the portable typing engine.

use std::{fmt, time::Duration};

use crate::AnimationThresholds;

/// Complete set of tunable V1 metric and session parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoreConfig {
    /// Duration used by the live WPM rolling window.
    pub rolling_window: Duration,
    /// Exponential moving-average factor in the interval `(0, 1]`.
    pub smoothing_factor: f64,
    /// Inactivity after which the overlay should be hidden.
    pub overlay_hide_after: Duration,
    /// Inactivity after which the current session is completed.
    pub session_end_after: Duration,
    /// Largest inter-key gap counted as active typing time.
    pub active_gap_limit: Duration,
    /// WPM boundaries used to select the visual intensity.
    pub animation_thresholds: AnimationThresholds,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            rolling_window: Duration::from_secs(10),
            smoothing_factor: 0.25,
            overlay_hide_after: Duration::from_secs(2),
            session_end_after: Duration::from_secs(30),
            active_gap_limit: Duration::from_secs(2),
            animation_thresholds: AnimationThresholds::default(),
        }
    }
}

impl CoreConfig {
    /// Validates all invariants before the configuration reaches an engine.
    pub fn validate(self) -> Result<Self, ConfigError> {
        if self.rolling_window.is_zero() {
            return Err(ConfigError::ZeroRollingWindow);
        }
        if !self.smoothing_factor.is_finite()
            || self.smoothing_factor <= 0.0
            || self.smoothing_factor > 1.0
        {
            return Err(ConfigError::InvalidSmoothingFactor);
        }
        if self.overlay_hide_after.is_zero() {
            return Err(ConfigError::ZeroOverlayDelay);
        }
        if self.session_end_after <= self.overlay_hide_after {
            return Err(ConfigError::SessionTimeoutNotAfterOverlay);
        }
        if self.active_gap_limit.is_zero() || self.active_gap_limit > self.session_end_after {
            return Err(ConfigError::InvalidActiveGapLimit);
        }
        self.animation_thresholds
            .validate()
            .map_err(|_| ConfigError::InvalidAnimationThresholds)?;
        Ok(self)
    }
}

/// Invalid core configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// A rolling window must cover some time.
    ZeroRollingWindow,
    /// EMA smoothing must be finite and in `(0, 1]`.
    InvalidSmoothingFactor,
    /// Overlay hiding must occur after a positive interval.
    ZeroOverlayDelay,
    /// Session timeout must be strictly longer than overlay timeout.
    SessionTimeoutNotAfterOverlay,
    /// Active-time gap must be positive and no longer than the session timeout.
    InvalidActiveGapLimit,
    /// Animation thresholds must be finite, non-negative and increasing.
    InvalidAnimationThresholds,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ZeroRollingWindow => "rolling window must be greater than zero",
            Self::InvalidSmoothingFactor => "smoothing factor must be finite and in (0, 1]",
            Self::ZeroOverlayDelay => "overlay delay must be greater than zero",
            Self::SessionTimeoutNotAfterOverlay => {
                "session timeout must be longer than the overlay delay"
            }
            Self::InvalidActiveGapLimit => {
                "active gap limit must be positive and no longer than the session timeout"
            }
            Self::InvalidAnimationThresholds => {
                "animation thresholds must be finite, non-negative and increasing"
            }
        };
        write!(formatter, "{message}")
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::AnimationThresholds;

    use super::{ConfigError, CoreConfig};

    #[test]
    fn v1_defaults_are_valid_and_documented_values() {
        let config = CoreConfig::default().validate().unwrap();
        assert_eq!(config.rolling_window, Duration::from_secs(10));
        assert_eq!(config.overlay_hide_after, Duration::from_secs(2));
        assert_eq!(config.session_end_after, Duration::from_secs(30));
        assert_eq!(config.smoothing_factor, 0.25);
    }

    #[test]
    fn rejects_each_invalid_relationship() {
        assert_eq!(
            CoreConfig {
                rolling_window: Duration::ZERO,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::ZeroRollingWindow)
        );
        assert_eq!(
            CoreConfig {
                smoothing_factor: f64::NAN,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidSmoothingFactor)
        );
        assert_eq!(
            CoreConfig {
                smoothing_factor: 0.0,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidSmoothingFactor)
        );
        assert_eq!(
            CoreConfig {
                smoothing_factor: 1.01,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidSmoothingFactor)
        );
        assert_eq!(
            CoreConfig {
                overlay_hide_after: Duration::ZERO,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::ZeroOverlayDelay)
        );
        assert_eq!(
            CoreConfig {
                session_end_after: Duration::from_secs(2),
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::SessionTimeoutNotAfterOverlay)
        );
        assert_eq!(
            CoreConfig {
                active_gap_limit: Duration::ZERO,
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidActiveGapLimit)
        );
        assert_eq!(
            CoreConfig {
                active_gap_limit: Duration::from_secs(31),
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidActiveGapLimit)
        );
        assert_eq!(
            CoreConfig {
                animation_thresholds: AnimationThresholds {
                    steady: 60.0,
                    fast: 60.0,
                    intense: 90.0,
                },
                ..CoreConfig::default()
            }
            .validate(),
            Err(ConfigError::InvalidAnimationThresholds)
        );
    }
}
