//! SQLite persistence adapter for TypePulse.
//!
//! Phase A establishes dependency direction only. The database schema and
//! migrations are introduced in Phase D.

/// Reports whether the storage crate is wired to the domain crate.
#[must_use]
pub const fn is_ready() -> bool {
    typepulse_core::is_ready()
}

#[cfg(test)]
mod tests {
    #[test]
    fn storage_depends_on_the_core_boundary() {
        assert!(super::is_ready());
    }
}
