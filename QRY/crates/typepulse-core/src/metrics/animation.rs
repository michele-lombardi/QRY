//! Mapping from displayed WPM to the four V1 animation intensities.

use std::fmt;

/// Visual intensity selected from the smoothed WPM value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnimationBand {
    /// From zero up to the steady threshold.
    Still,
    /// Normal movement.
    Steady,
    /// Fast movement.
    Fast,
    /// Highest continuous intensity.
    Intense,
}

impl AnimationBand {
    /// Stable string for application DTO conversion.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Still => "still",
            Self::Steady => "steady",
            Self::Fast => "fast",
            Self::Intense => "intense",
        }
    }
}

/// Increasing WPM thresholds for the four animation bands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationThresholds {
    /// First WPM value in the steady band.
    pub steady: f64,
    /// First WPM value in the fast band.
    pub fast: f64,
    /// First WPM value in the intense band.
    pub intense: f64,
}

impl Default for AnimationThresholds {
    fn default() -> Self {
        Self {
            steady: 30.0,
            fast: 60.0,
            intense: 90.0,
        }
    }
}

impl AnimationThresholds {
    /// Validates finite, non-negative, strictly increasing boundaries.
    pub fn validate(self) -> Result<Self, AnimationThresholdError> {
        if self.steady.is_finite()
            && self.fast.is_finite()
            && self.intense.is_finite()
            && self.steady >= 0.0
            && self.steady < self.fast
            && self.fast < self.intense
        {
            Ok(self)
        } else {
            Err(AnimationThresholdError)
        }
    }

    /// Selects the visual band for a WPM value.
    #[must_use]
    pub fn band_for(self, wpm: f64) -> AnimationBand {
        let wpm = if wpm.is_finite() { wpm.max(0.0) } else { 0.0 };
        if wpm >= self.intense {
            AnimationBand::Intense
        } else if wpm >= self.fast {
            AnimationBand::Fast
        } else if wpm >= self.steady {
            AnimationBand::Steady
        } else {
            AnimationBand::Still
        }
    }
}

/// Invalid animation thresholds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnimationThresholdError;

impl fmt::Display for AnimationThresholdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "animation thresholds must be finite and strictly increasing"
        )
    }
}

impl std::error::Error for AnimationThresholdError {}

#[cfg(test)]
mod tests {
    use super::{AnimationBand, AnimationThresholds};

    #[test]
    fn exact_v1_boundaries_select_the_higher_band() {
        let thresholds = AnimationThresholds::default();
        for (wpm, expected) in [
            (0.0, AnimationBand::Still),
            (29.999, AnimationBand::Still),
            (30.0, AnimationBand::Steady),
            (59.999, AnimationBand::Steady),
            (60.0, AnimationBand::Fast),
            (89.999, AnimationBand::Fast),
            (90.0, AnimationBand::Intense),
        ] {
            assert_eq!(thresholds.band_for(wpm), expected);
        }
    }

    #[test]
    fn pathological_values_are_safe() {
        let thresholds = AnimationThresholds::default();
        assert_eq!(thresholds.band_for(f64::NAN), AnimationBand::Still);
        assert_eq!(thresholds.band_for(f64::NEG_INFINITY), AnimationBand::Still);
        assert_eq!(thresholds.band_for(f64::INFINITY), AnimationBand::Still);
    }
}
