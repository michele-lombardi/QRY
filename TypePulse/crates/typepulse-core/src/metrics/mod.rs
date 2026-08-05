//! Live typing metrics independent from application and operating system.

mod animation;
mod rolling_wpm;
mod smoothing;

pub use animation::{AnimationBand, AnimationThresholdError, AnimationThresholds};
pub(crate) use rolling_wpm::RollingWpm;
pub(crate) use smoothing::ExponentialSmoother;
