//! Adaptive warm-up and fixed-lookback rolling WPM calculation.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const MIN_ESTIMATE_SPAN: Duration = Duration::from_millis(250);
const LIVE_WARM_UP_FLOOR: Duration = Duration::from_secs(1);
const RECORD_MIN_SPAN: Duration = Duration::from_secs(3);
const MAX_LIVE_WPM: f64 = 300.0;

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

    /// Whether at least one reliable inter-activity interval is available.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.estimate().is_some()
    }

    /// Whether the observation is long enough for persisted statistics and records.
    #[must_use]
    pub fn is_record_ready(&self) -> bool {
        self.observed_span()
            .is_some_and(|span| span >= RECORD_MIN_SPAN)
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
        self.estimate().unwrap_or(0.0)
    }

    fn estimate(&self) -> Option<f64> {
        let interval_count = self.activities.len().checked_sub(1)?;
        let observed = self.observed_span()?;
        if interval_count == 0 || observed < MIN_ESTIMATE_SPAN {
            return None;
        }

        let estimated_words = interval_count as f64 / 5.0;
        let observed_minutes = observed.max(LIVE_WARM_UP_FLOOR).as_secs_f64() / 60.0;
        let wpm = estimated_words / observed_minutes;
        wpm.is_finite().then(|| wpm.clamp(0.0, MAX_LIVE_WPM))
    }

    fn observed_span(&self) -> Option<Duration> {
        self.activities
            .back()?
            .checked_duration_since(*self.activities.front()?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{RollingWpm, MAX_LIVE_WPM};

    #[test]
    fn empty_window_is_zero() {
        let now = Instant::now();
        assert_eq!(RollingWpm::new(Duration::from_secs(10)).at(now), 0.0);
    }

    #[test]
    fn warm_up_becomes_reactive_after_a_short_reliable_span() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        assert_eq!(metric.record(origin), 0.0);
        assert_eq!(metric.record(origin + Duration::from_millis(200)), 0.0);
        assert_eq!(metric.record(origin + Duration::from_millis(400)), 24.0);
        assert!(metric.is_ready());
        assert!(!metric.is_record_ready());
        metric.record(origin + Duration::from_secs(3));
        assert!(metric.is_record_ready());
    }

    #[test]
    fn steady_five_characters_per_second_is_sixty_wpm() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        for offset_ms in (0..=50).map(|index| index * 200) {
            metric.record(origin + Duration::from_millis(offset_ms));
        }
        assert_eq!(metric.activity_count(), 50);
        assert!((metric.at(origin + Duration::from_secs(10)) - 60.0).abs() < 1e-9);
    }

    #[test]
    fn slow_typing_expires_from_the_window() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        metric.record(origin);
        assert!((metric.record(origin + Duration::from_secs(5)) - 2.4).abs() < 1e-9);
        assert_eq!(metric.at(origin + Duration::from_secs(10)), 0.0);
        assert_eq!(metric.at(origin + Duration::from_secs(15)), 0.0);
    }

    #[test]
    fn simultaneous_burst_is_zero_and_later_extreme_rate_is_capped() {
        let origin = Instant::now();
        let mut metric = RollingWpm::new(Duration::from_secs(10));
        let value = (0..1_000).fold(0.0, |_, _| metric.record(origin));
        assert!(value.is_finite());
        assert_eq!(value, 0.0);
        assert_eq!(
            metric.record(origin + Duration::from_millis(300)),
            MAX_LIVE_WPM
        );
    }
}
