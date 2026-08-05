//! macOS platform adapter for TypePulse.
//!
//! Phase A establishes the platform boundary. Input Monitoring permission and
//! the global event tap are intentionally left for the Phase B spike.

/// Reports whether the adapter crate can see the portable core boundary.
#[must_use]
pub const fn is_ready() -> bool {
    typepulse_core::is_ready()
}

#[cfg(test)]
mod tests {
    #[test]
    fn macos_adapter_depends_on_the_core_boundary() {
        assert!(super::is_ready());
    }
}
