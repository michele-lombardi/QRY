//! Fixed-duration rolling WPM calculation.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Counts activity in a fixed rolling window using five characters per word.
#[derive(Clone, Debug)]
pub struct RollingWpm {
    window: Duration,
    activities: VecDeque<Instant>,
}

impl RollingWpm {
    /// Creates a rolling metric. `window` must be non-zero.
    #[must_use]
    pub fn new(window: Duration) -> Self {
        debug_assert!(!window.is_zero());
        Self {
            window,
            activities: VecDeque::new(),
        }
    }

    /// Adds one activity and returns WPM at that instant.
    pub fn record(&mut self, occurred_at: Instant) -> f64 {
        self.prune(occurred_at);
        self.activities.push_back(occurred_at);
        self.value()
    }

    /// Removes expired activity and returns WPM for `now`.
    pub fn at(&mut self, now: Instant) -> f64 {
        self.prune(now);
        self.value()
    }

    /// Clears all ephemeral activity timestamps.
    pub fn reset(&mut self) {
        self.activities.clear();
    }

    /// Number of activities currently held in the rolling window.
    #[cfg(test)]
    #[must_use]
    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }

    fn prune(&mut self, now: Instant) {
        let Some(cutoff) = now.checked_sub(self.window) else {
            return;
        };
        while self
            .activities
            .front()
            .is_some_and(|instant| *instant <= cutoff)
        {
            self.activities.pop_front();
        }
    }

    fn value(&self) -> f64 {
        let words = self.activities.len() as f64 / 5.0;
        let window_minutes = self.window.as_secs_f64() / 60.0;
        let wpm = words / window_minutes;
        if wpm.is_finite() {
            wpm.max(0.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::RollingWpm;

    #[test]
    fn empty_window_is_zero() {
        let now = Instant::now();
        assert_eq!(RollingWpm::new(Duration::from_secs(10)).at(now), 0.0);
    }

    #[test]
    fn fifty_characters_in_ten_seconds_is_sixty_wpm() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        for offset_ms in (0..50).map(|index| index * 190) {
            metric.record(origin + Duration::from_millis(offset_ms));
        }
        assert_eq!(metric.activity_count(), 50);
        assert!((metric.at(origin + Duration::from_millis(9_500)) - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn slow_typing_expires_from_the_window() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        metric.record(origin);
        metric.record(origin + Duration::from_secs(5));
        assert!((metric.at(origin + Duration::from_secs(10)) - 1.2).abs() < 1e-9);
        assert_eq!(metric.at(origin + Duration::from_secs(15)), 0.0);
    }

    #[test]
    fn burst_is_finite_and_uses_the_fixed_window() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        let value = (0..1_000).fold(0.0, |_, _| metric.record(origin));
        assert!(value.is_finite());
        assert_eq!(value, 1_200.0);
    }
}
