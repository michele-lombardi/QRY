//! Injectable monotonic clocks for production and deterministic tests.

use std::{
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Source of monotonic time used by the typing engine.
pub trait Clock: Clone + Send + Sync + 'static {
    /// Returns the current monotonic instant.
    fn now(&self) -> Instant;
}

/// Production clock backed by [`Instant::now`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Thread-safe clock whose time advances only when explicitly requested.
#[derive(Clone, Debug)]
pub struct ManualClock {
    now: Arc<Mutex<Instant>>,
}

impl ManualClock {
    /// Creates a manual clock at the supplied monotonic instant.
    #[must_use]
    pub fn new(now: Instant) -> Self {
        Self {
            now: Arc::new(Mutex::new(now)),
        }
    }

    /// Advances the clock and returns its new value.
    pub fn advance(&self, duration: Duration) -> Result<Instant, ClockError> {
        let mut now = self.now.lock().unwrap_or_else(|error| error.into_inner());
        *now = now
            .checked_add(duration)
            .ok_or(ClockError::InstantOverflow)?;
        Ok(*now)
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap_or_else(|error| error.into_inner())
    }
}

/// Failure while manipulating a deterministic clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockError {
    /// Advancing the instant exceeded the platform representation.
    InstantOverflow,
}

impl fmt::Display for ClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "manual clock instant overflow")
    }
}

impl std::error::Error for ClockError {}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Clock, ManualClock};

    #[test]
    fn manual_clock_advances_without_sleeping() {
        let origin = Instant::now();
        let clock = ManualClock::new(origin);
        clock.advance(Duration::from_secs(42)).unwrap();
        assert_eq!(clock.now(), origin + Duration::from_secs(42));
    }
}
