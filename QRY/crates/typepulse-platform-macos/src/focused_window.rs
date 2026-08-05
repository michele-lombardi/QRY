//! Privacy-minimized focused-window geometry for macOS display selection.
//!
//! This adapter reads only the focused window's position and size. It never
//! requests its title, role, value, owning application, or written content.

/// A point in the global Core Graphics display coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    /// Horizontal global display coordinate.
    pub x: f64,
    /// Vertical global display coordinate.
    pub y: f64,
}

/// Returns the center of the focused window when Accessibility access and
/// usable geometry are available.
#[must_use]
pub fn focused_window_center() -> Option<ScreenPoint> {
    platform::focused_window_center()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::{ffi::c_void, ptr};

    use core_foundation::{
        base::{CFType, CFTypeRef, TCFType},
        string::{CFString, CFStringRef},
    };
    use core_graphics::geometry::{CGPoint, CGSize};

    use super::ScreenPoint;
    use crate::permissions::accessibility_permission_status;
    use crate::PermissionStatus;

    type AXUIElementRef = *const c_void;
    type AXValueRef = *const c_void;
    type AXError = i32;
    type AXValueType = u32;

    const AX_ERROR_SUCCESS: AXError = 0;
    const AX_VALUE_TYPE_POINT: AXValueType = 1;
    const AX_VALUE_TYPE_SIZE: AXValueType = 2;
    const MESSAGING_TIMEOUT_SECONDS: f32 = 0.2;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXUIElementSetMessagingTimeout(
            element: AXUIElementRef,
            timeout_in_seconds: f32,
        ) -> AXError;
        fn AXValueGetValue(
            value: AXValueRef,
            value_type: AXValueType,
            value_ptr: *mut c_void,
        ) -> u8;
    }

    pub(super) fn focused_window_center() -> Option<ScreenPoint> {
        if accessibility_permission_status() != PermissionStatus::Granted {
            return None;
        }

        // SAFETY: all returned Core Foundation values are checked for null and
        // wrapped under the create rule. Attribute values stay owned for the
        // duration of every AX call and are released by `CFType` on return.
        unsafe {
            let system_ref = AXUIElementCreateSystemWide();
            if system_ref.is_null() {
                return None;
            }
            let system = CFType::wrap_under_create_rule(system_ref.cast());
            let _ = AXUIElementSetMessagingTimeout(
                system.as_CFTypeRef().cast(),
                MESSAGING_TIMEOUT_SECONDS,
            );

            let application = copy_attribute(system.as_CFTypeRef().cast(), "AXFocusedApplication")?;
            let window = copy_attribute(application.as_CFTypeRef().cast(), "AXFocusedWindow")?;
            let position = copy_attribute(window.as_CFTypeRef().cast(), "AXPosition")?;
            let size = copy_attribute(window.as_CFTypeRef().cast(), "AXSize")?;

            let mut origin = CGPoint::new(0.0, 0.0);
            let mut dimensions = CGSize::new(0.0, 0.0);
            if AXValueGetValue(
                position.as_CFTypeRef().cast(),
                AX_VALUE_TYPE_POINT,
                ptr::from_mut(&mut origin).cast(),
            ) == 0
                || AXValueGetValue(
                    size.as_CFTypeRef().cast(),
                    AX_VALUE_TYPE_SIZE,
                    ptr::from_mut(&mut dimensions).cast(),
                ) == 0
            {
                return None;
            }

            center_from_geometry(origin, dimensions)
        }
    }

    unsafe fn copy_attribute(element: AXUIElementRef, name: &str) -> Option<CFType> {
        let attribute = CFString::new(name);
        let mut value: CFTypeRef = ptr::null();
        // SAFETY: `element` is a live AX object, `attribute` remains alive for
        // the call, and the out pointer is valid. Success follows the create rule.
        if unsafe {
            AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
        } != AX_ERROR_SUCCESS
            || value.is_null()
        {
            return None;
        }
        // SAFETY: a successful AX copy returns an owned Core Foundation object.
        Some(unsafe { CFType::wrap_under_create_rule(value) })
    }

    fn center_from_geometry(origin: CGPoint, size: CGSize) -> Option<ScreenPoint> {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || !size.width.is_finite()
            || !size.height.is_finite()
            || size.width <= 0.0
            || size.height <= 0.0
        {
            return None;
        }

        let center = ScreenPoint {
            x: origin.x + size.width / 2.0,
            y: origin.y + size.height / 2.0,
        };
        (center.x.is_finite() && center.y.is_finite()).then_some(center)
    }

    #[cfg(test)]
    mod tests {
        use core_graphics::geometry::{CGPoint, CGSize};

        use super::{center_from_geometry, ScreenPoint};

        #[test]
        fn center_uses_only_window_geometry() {
            assert_eq!(
                center_from_geometry(CGPoint::new(-1_920.0, 40.0), CGSize::new(1_200.0, 800.0)),
                Some(ScreenPoint {
                    x: -1_320.0,
                    y: 440.0
                })
            );
        }

        #[test]
        fn invalid_geometry_has_no_display_target() {
            assert_eq!(
                center_from_geometry(CGPoint::new(10.0, 20.0), CGSize::new(0.0, 100.0)),
                None
            );
            assert_eq!(
                center_from_geometry(CGPoint::new(f64::NAN, 20.0), CGSize::new(100.0, 100.0)),
                None
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::ScreenPoint;

    pub(super) const fn focused_window_center() -> Option<ScreenPoint> {
        None
    }
}
