//! Fixed-window WPM records derived only from privacy-safe activity timestamps.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Keeps enough aggregate timing information for 30 and 60 second records.
#[derive(Clone, Debug)]
pub(crate) struct SustainedWpm {
    maximum_window: Duration,
    activities: VecDeque<Instant>,
}

impl SustainedWpm {
    pub(crate) fn new(maximum_window: Duration) -> Self {
        Self {
            maximum_window,
            activities: VecDeque::new(),
        }
    }

    pub(crate) fn record(&mut self, occurred_at: Instant) {
        self.activities.push_back(occurred_at);
        self.prune(occurred_at);
    }

    /// Returns WPM only after the session has covered the complete window.
    pub(crate) fn at(
        &self,
        now: Instant,
        session_started_at: Instant,
        window: Duration,
    ) -> Option<f64> {
        if window.is_zero()
            || now
                .checked_duration_since(session_started_at)
                .unwrap_or_default()
                < window
        {
            return None;
        }

        let cutoff = now.checked_sub(window).unwrap_or(session_started_at);
        let activity_count = self
            .activities
            .iter()
            .filter(|instant| **instant > cutoff && **instant <= now)
            .count();
        Some(activity_count as f64 / 5.0 / (window.as_secs_f64() / 60.0))
    }

    pub(crate) fn reset(&mut self) {
        self.activities.clear();
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.maximum_window);
        while self
            .activities
            .front()
            .is_some_and(|instant| cutoff.is_some_and(|cutoff| *instant <= cutoff))
        {
            self.activities.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::SustainedWpm;

    #[test]
    fn requires_a_complete_window_and_uses_only_activities_inside_it() {
        let origin = Instant::now();
        let mut metric = SustainedWpm::new(Duration::from_secs(60));
        for second in 0..=60 {
            metric.record(origin + Duration::from_secs(second));
        }

        assert_eq!(
            metric.at(
                origin + Duration::from_secs(29),
                origin,
                Duration::from_secs(30)
            ),
            None
        );
        assert_eq!(
            metric.at(
                origin + Duration::from_secs(60),
                origin,
                Duration::from_secs(30)
            ),
            Some(12.0)
        );
        assert_eq!(
            metric.at(
                origin + Duration::from_secs(60),
                origin,
                Duration::from_secs(60)
            ),
            Some(12.0)
        );
    }
}
