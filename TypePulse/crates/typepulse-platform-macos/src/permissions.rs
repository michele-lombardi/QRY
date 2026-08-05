//! Input Monitoring permission checks and System Settings navigation.

use std::{fmt, io};

/// Current ability to listen to input events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionStatus {
    /// macOS reports that listen-event access is available.
    Granted,
    /// macOS reports that listen-event access is not available.
    Denied,
    /// The current platform cannot evaluate macOS Input Monitoring access.
    Unknown,
}

impl PermissionStatus {
    /// Stable string used by the application DTO layer.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::Unknown => "unknown",
        }
    }
}

/// Error returned while opening the macOS privacy settings.
#[derive(Debug)]
pub enum PermissionError {
    /// This operation is only available on macOS.
    UnsupportedPlatform,
    /// Starting the System Settings helper failed.
    Launch(io::Error),
    /// The helper process returned an unsuccessful exit status.
    UnsuccessfulLaunch,
}

impl fmt::Display for PermissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(formatter, "Input Monitoring is macOS-only"),
            Self::Launch(error) => write!(formatter, "failed to open System Settings: {error}"),
            Self::UnsuccessfulLaunch => {
                write!(formatter, "System Settings did not accept the privacy URL")
            }
        }
    }
}

impl std::error::Error for PermissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Launch(error) => Some(error),
            Self::UnsupportedPlatform | Self::UnsuccessfulLaunch => None,
        }
    }
}

/// Deep link for Privacy & Security → Input Monitoring.
#[must_use]
pub const fn input_monitoring_settings_url() -> &'static str {
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
}

/// Reads the current Input Monitoring permission without showing a prompt.
#[must_use]
pub fn input_permission_status() -> PermissionStatus {
    platform::input_permission_status()
}

/// Requests listen-event access and returns the resulting status.
#[must_use]
pub fn request_input_permission() -> PermissionStatus {
    platform::request_input_permission()
}

/// Opens the Input Monitoring section of System Settings.
pub fn open_input_monitoring_settings() -> Result<(), PermissionError> {
    platform::open_input_monitoring_settings()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;

    use objc2_core_graphics::{CGPreflightListenEventAccess, CGRequestListenEventAccess};

    use super::{input_monitoring_settings_url, PermissionError, PermissionStatus};

    pub(super) fn input_permission_status() -> PermissionStatus {
        if CGPreflightListenEventAccess() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        }
    }

    pub(super) fn request_input_permission() -> PermissionStatus {
        if CGRequestListenEventAccess() {
            PermissionStatus::Granted
        } else {
            input_permission_status()
        }
    }

    pub(super) fn open_input_monitoring_settings() -> Result<(), PermissionError> {
        let status = Command::new("/usr/bin/open")
            .arg(input_monitoring_settings_url())
            .status()
            .map_err(PermissionError::Launch)?;

        if status.success() {
            Ok(())
        } else {
            Err(PermissionError::UnsuccessfulLaunch)
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{PermissionError, PermissionStatus};

    pub(super) const fn input_permission_status() -> PermissionStatus {
        PermissionStatus::Unknown
    }

    pub(super) const fn request_input_permission() -> PermissionStatus {
        PermissionStatus::Unknown
    }

    pub(super) const fn open_input_monitoring_settings() -> Result<(), PermissionError> {
        Err(PermissionError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::{input_monitoring_settings_url, PermissionStatus};

    #[test]
    fn permission_status_strings_are_stable() {
        assert_eq!(PermissionStatus::Granted.as_str(), "granted");
        assert_eq!(PermissionStatus::Denied.as_str(), "denied");
        assert_eq!(PermissionStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn settings_url_targets_input_monitoring() {
        assert!(input_monitoring_settings_url().ends_with("Privacy_ListenEvent"));
    }
}
