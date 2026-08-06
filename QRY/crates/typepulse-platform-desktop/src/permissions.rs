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
            Self::UnsupportedPlatform => {
                write!(formatter, "macOS privacy settings are unavailable")
            }
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

/// Deep link for Privacy & Security → Accessibility.
#[must_use]
pub const fn accessibility_settings_url() -> &'static str {
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
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

/// Reads the current Accessibility permission without showing a prompt.
#[must_use]
pub fn accessibility_permission_status() -> PermissionStatus {
    platform::accessibility_permission_status()
}

/// Asks macOS to show its Accessibility consent prompt when access is absent.
#[must_use]
pub fn request_accessibility_permission() -> PermissionStatus {
    platform::request_accessibility_permission()
}

/// Opens the Accessibility section of System Settings.
pub fn open_accessibility_settings() -> Result<(), PermissionError> {
    platform::open_accessibility_settings()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;
    use std::ptr;

    use core_foundation::{
        base::TCFType,
        boolean::CFBoolean,
        dictionary::{CFDictionary, CFDictionaryRef},
        string::CFStringRef,
    };
    use objc2_core_graphics::{CGPreflightListenEventAccess, CGRequestListenEventAccess};

    use super::{
        accessibility_settings_url, input_monitoring_settings_url, PermissionError,
        PermissionStatus,
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

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
        open_settings_url(input_monitoring_settings_url())
    }

    pub(super) fn accessibility_permission_status() -> PermissionStatus {
        // SAFETY: a null options dictionary is explicitly supported by Apple
        // and performs a read-only trust check without displaying a prompt.
        if unsafe { AXIsProcessTrustedWithOptions(ptr::null()) } != 0 {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        }
    }

    pub(super) fn request_accessibility_permission() -> PermissionStatus {
        // SAFETY: the exported key is a process-lifetime CFString. The wrapped
        // value retains it while constructing a valid CFDictionary.
        unsafe {
            let prompt_key =
                core_foundation::string::CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
            let options = CFDictionary::from_CFType_pairs(&[(prompt_key, CFBoolean::true_value())]);
            let _ = AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
        }
        accessibility_permission_status()
    }

    pub(super) fn open_accessibility_settings() -> Result<(), PermissionError> {
        open_settings_url(accessibility_settings_url())
    }

    fn open_settings_url(url: &str) -> Result<(), PermissionError> {
        let status = Command::new("/usr/bin/open")
            .arg(url)
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

    pub(super) const fn accessibility_permission_status() -> PermissionStatus {
        PermissionStatus::Unknown
    }

    pub(super) const fn request_accessibility_permission() -> PermissionStatus {
        PermissionStatus::Unknown
    }

    pub(super) const fn open_accessibility_settings() -> Result<(), PermissionError> {
        Err(PermissionError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::{accessibility_settings_url, input_monitoring_settings_url, PermissionStatus};

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

    #[test]
    fn settings_url_targets_accessibility() {
        assert!(accessibility_settings_url().ends_with("Privacy_Accessibility"));
    }
}
