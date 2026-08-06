//! Desktop permission, global-input, and focused-display boundary for QRY.
//!
//! The input boundary deliberately omits key codes, characters and active-app
//! identity. Consumers receive only [`typepulse_core::TypingActivity`]. The
//! separate placement boundary exposes one ephemeral window-center point and
//! never exposes title, application identity or content.
//!
//! Native implementations stay behind target-gated private modules so the
//! application composition layer consumes one stable API on every desktop OS.

mod focused_window;
mod monitor;
mod permissions;

#[cfg(target_os = "macos")]
mod event_filter;

#[cfg(any(windows, test))]
mod event_filter_windows;

pub use focused_window::{focused_window_center, ScreenPoint};
pub use monitor::{
    ActivityReceiver, KeyboardMonitor, MonitorConfig, MonitorError, MonitorMetricsSnapshot,
    MonitorRunState,
};
pub use permissions::{
    accessibility_permission_status, accessibility_settings_url, input_monitoring_settings_url,
    input_permission_status, open_accessibility_settings, open_input_monitoring_settings,
    platform_capabilities, request_accessibility_permission, request_input_permission,
    PermissionError, PermissionStatus, PlatformCapabilities,
};

/// Reports whether the adapter crate can see the portable core boundary.
#[must_use]
pub const fn is_ready() -> bool {
    typepulse_core::is_ready()
}

#[cfg(test)]
mod tests {
    #[test]
    fn desktop_adapter_depends_on_the_core_boundary() {
        assert!(super::is_ready());
    }
}
