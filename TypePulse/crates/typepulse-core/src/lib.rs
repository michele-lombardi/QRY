//! Platform-independent domain logic for TypePulse.
//!
//! Phase A establishes the crate boundary. Typing metrics and session logic are
//! intentionally implemented in Phase C.

mod activity;

pub use activity::TypingActivity;

/// Reports whether the core crate is available to the application workspace.
#[must_use]
pub const fn is_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_crate_is_available() {
        assert!(super::is_ready());
    }
}
