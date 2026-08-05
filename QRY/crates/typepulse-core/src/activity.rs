//! Privacy-preserving activity signal shared with platform adapters.

use std::time::Instant;

/// A typing-like occurrence without key identity or written content.
///
/// `Instant` is monotonic and intentionally not serializable. Persistence and
/// frontend layers receive aggregates in later phases, never this raw signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypingActivity {
    occurred_at: Instant,
}

impl TypingActivity {
    /// Creates an activity at a supplied monotonic instant.
    #[must_use]
    pub const fn at(occurred_at: Instant) -> Self {
        Self { occurred_at }
    }

    /// Returns the monotonic instant at which activity occurred.
    #[must_use]
    pub const fn occurred_at(self) -> Instant {
        self.occurred_at
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::TypingActivity;

    #[test]
    fn exposes_only_the_monotonic_occurrence_time() {
        let now = Instant::now();
        let activity = TypingActivity::at(now);

        assert_eq!(activity.occurred_at(), now);
    }
}
