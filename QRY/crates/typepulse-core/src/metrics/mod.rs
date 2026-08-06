//! Live typing metrics independent from application and operating system.

mod animation;
mod rolling_wpm;
mod smoothing;
mod sustained_wpm;

pub use animation::{AnimationBand, AnimationThresholdError, AnimationThresholds};
pub(crate) use rolling_wpm::RollingWpm;
pub(crate) use smoothing::ExponentialSmoother;
pub(crate) use sustained_wpm::SustainedWpm;
