//! Configurable exponential smoothing for the displayed WPM value.

/// Exponential moving average initialized from its first sample.
#[derive(Clone, Copy, Debug)]
pub struct ExponentialSmoother {
    factor: f64,
    value: Option<f64>,
}

impl ExponentialSmoother {
    /// Creates a smoother with a pre-validated factor in `(0, 1]`.
    #[must_use]
    pub const fn new(factor: f64) -> Self {
        Self {
            factor,
            value: None,
        }
    }

    /// Incorporates one non-negative sample and returns the new value.
    pub fn update(&mut self, sample: f64) -> f64 {
        let sample = sanitize(sample);
        let next = self
            .value
            .map_or(sample, |current| current + self.factor * (sample - current));
        self.value = Some(sanitize(next));
        self.value.unwrap_or(0.0)
    }

    /// Current smoothed value, or zero before the first sample.
    #[must_use]
    pub fn value(self) -> f64 {
        self.value.unwrap_or(0.0)
    }

    /// Clears the moving average between sessions.
    pub fn reset(&mut self) {
        self.value = None;
    }
}

fn sanitize(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::ExponentialSmoother;

    #[test]
    fn first_value_has_no_startup_lag() {
        let mut smoother = ExponentialSmoother::new(0.25);
        assert_eq!(smoother.update(60.0), 60.0);
    }

    #[test]
    fn configured_factor_dampens_a_jump() {
        let mut smoother = ExponentialSmoother::new(0.25);
        smoother.update(40.0);
        assert_eq!(smoother.update(80.0), 50.0);
        assert_eq!(smoother.update(80.0), 57.5);
    }

    #[test]
    fn invalid_samples_never_escape() {
        let mut smoother = ExponentialSmoother::new(0.25);
        assert_eq!(smoother.update(f64::NAN), 0.0);
        assert_eq!(smoother.update(f64::INFINITY), 0.0);
        assert_eq!(smoother.update(-10.0), 0.0);
    }

    #[test]
    fn reset_forgets_the_previous_session() {
        let mut smoother = ExponentialSmoother::new(0.25);
        smoother.update(100.0);
        smoother.reset();
        assert_eq!(smoother.value(), 0.0);
        assert_eq!(smoother.update(20.0), 20.0);
    }
}
