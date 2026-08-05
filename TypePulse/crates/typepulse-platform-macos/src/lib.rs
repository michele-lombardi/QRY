//! macOS permission and global-input adapter for TypePulse.
//!
//! The public boundary deliberately omits key codes, characters, active apps,
//! and window metadata. Consumers receive only [`typepulse_core::TypingActivity`].

mod monitor;
mod permissions;

#[cfg(target_os = "macos")]
mod event_filter;

pub use monitor::{
    ActivityReceiver, KeyboardMonitor, MonitorConfig, MonitorError, MonitorMetricsSnapshot,
    MonitorRunState,
};
pub use permissions::{
    input_monitoring_settings_url, input_permission_status, open_input_monitoring_settings,
    request_input_permission, PermissionError, PermissionStatus,
};

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
