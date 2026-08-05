//! Non-interactive live-metric overlay window and display placement controller.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tauri::{
    App, AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Runtime, State, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};
use typepulse_core::{AppPreferences, OverlayContent, OverlayPosition, OverlaySize};

use crate::app_state::DiagnosticState;

pub(crate) const OVERLAY_LABEL: &str = "overlay";
const OVERLAY_EVENT: &str = "typepulse://overlay-state";
const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const FOCUSED_DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
const UPDATE_INTERVAL: Duration = Duration::from_millis(50);
const FADE_OUT_DURATION: Duration = Duration::from_millis(180);
const SCREEN_MARGIN_LOGICAL: f64 = 20.0;

/// Preferences exposed to the settings UI without any input information.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OverlayPreferenceDto {
    enabled: bool,
    position: &'static str,
    size: &'static str,
    content: &'static str,
}

impl From<AppPreferences> for OverlayPreferenceDto {
    fn from(preferences: AppPreferences) -> Self {
        Self {
            enabled: preferences.overlay_enabled,
            position: preferences.overlay_position.as_str(),
            size: preferences.overlay_size.as_str(),
            content: preferences.overlay_content.as_str(),
        }
    }
}

/// Validated overlay preference mutation received from the main window.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OverlayPreferenceInput {
    enabled: bool,
    position: String,
    size: String,
    content: String,
}

/// Complete frontend presentation state emitted by the controller.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayEventDto {
    visible: bool,
    displayed_wpm: f64,
    animation_band: &'static str,
    behavior: &'static str,
    content: &'static str,
    size: &'static str,
    celebration_sequence: u64,
}

/// Shared overlay configuration and lifecycle flag.
#[derive(Clone)]
pub(crate) struct OverlayRuntime {
    preferences: Arc<Mutex<AppPreferences>>,
    stopped: Arc<AtomicBool>,
}

impl OverlayRuntime {
    fn new(preferences: AppPreferences) -> Self {
        Self {
            preferences: Arc::new(Mutex::new(preferences)),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    fn preferences(&self) -> AppPreferences {
        *self
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn update(&self, preferences: AppPreferences) {
        *self
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = preferences;
    }

    pub(crate) fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }
}

/// Builds the hidden overlay and starts its lightweight presentation controller.
pub(crate) fn configure(app: &mut App, preferences: AppPreferences) -> tauri::Result<()> {
    let initial_size = dimensions(preferences.overlay_size);
    let window =
        WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay.html".into()))
            .title("TypePulse live rhythm")
            .inner_size(initial_size.0, initial_size.1)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .skip_taskbar(true)
            .focusable(false)
            .visible(false)
            .build()?;
    window.set_ignore_cursor_events(true)?;
    position_window(app.handle(), &window, preferences)?;

    let runtime = OverlayRuntime::new(preferences);
    app.manage(runtime.clone());
    let handle = app.handle().clone();
    std::thread::Builder::new()
        .name("typepulse-overlay-controller".into())
        .spawn(move || run_controller(handle, runtime))
        .map_err(tauri::Error::Io)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn overlay_preference(
    state: State<'_, DiagnosticState>,
) -> Result<OverlayPreferenceDto, String> {
    state.load_preferences().map(Into::into)
}

#[tauri::command]
pub(crate) fn set_overlay_preference(
    preference: OverlayPreferenceInput,
    state: State<'_, DiagnosticState>,
    runtime: State<'_, OverlayRuntime>,
) -> Result<OverlayPreferenceDto, String> {
    let position = OverlayPosition::from_stored(&preference.position)
        .ok_or_else(|| "invalid overlay position".to_string())?;
    let size = OverlaySize::from_stored(&preference.size)
        .ok_or_else(|| "invalid overlay size".to_string())?;
    let content = OverlayContent::from_stored(&preference.content)
        .ok_or_else(|| "invalid overlay content".to_string())?;
    let mut preferences = state.load_preferences()?;
    preferences.overlay_enabled = preference.enabled;
    preferences.overlay_position = position;
    preferences.overlay_size = size;
    preferences.overlay_content = content;
    state.save_preferences(preferences)?;
    runtime.update(preferences);
    Ok(preferences.into())
}

fn run_controller<R: Runtime>(app: AppHandle<R>, runtime: OverlayRuntime) {
    let mut presented = false;
    let mut hide_after: Option<Instant> = None;
    let mut last_payload: Option<OverlayEventDto> = None;
    let mut last_tray_status: Option<(bool, u64)> = None;
    let mut last_preferences = runtime.preferences();
    let mut next_display_refresh = Instant::now();
    let mut next_focused_display_refresh = Instant::now();
    let mut positioned_activity_count = 0;

    while !runtime.stopped.load(Ordering::Acquire) {
        let Some(window) = app.get_webview_window(OVERLAY_LABEL) else {
            return;
        };
        let preferences = runtime.preferences();
        let snapshot = app.state::<DiagnosticState>().snapshot();
        let should_present =
            preferences.overlay_enabled && snapshot.live_metrics.phase.overlay_visible();
        let now = Instant::now();

        let preferences_changed = preferences.overlay_size != last_preferences.overlay_size
            || preferences.overlay_position != last_preferences.overlay_position
            || preferences.overlay_enabled != last_preferences.overlay_enabled;
        let first_presentation = should_present && !presented;
        let typing_moved_on = should_present
            && snapshot.total_activities != positioned_activity_count
            && now >= next_focused_display_refresh;
        if preferences_changed
            || first_presentation
            || typing_moved_on
            || now >= next_display_refresh
        {
            if let Err(error) = position_window(&app, &window, preferences) {
                app.state::<DiagnosticState>()
                    .record_runtime_error(format!("overlay placement failed: {error}"));
            }
            last_preferences = preferences;
            next_display_refresh = now + DISPLAY_REFRESH_INTERVAL;
            if first_presentation || typing_moved_on {
                positioned_activity_count = snapshot.total_activities;
                next_focused_display_refresh = now + FOCUSED_DISPLAY_REFRESH_INTERVAL;
            }
        }

        if should_present {
            hide_after = None;
            if !presented {
                if let Err(error) = window.show() {
                    app.state::<DiagnosticState>()
                        .record_runtime_error(format!("overlay presentation failed: {error}"));
                } else {
                    presented = true;
                }
            }
        } else if presented && hide_after.is_none() {
            hide_after = Some(now + FADE_OUT_DURATION);
        }

        let payload = OverlayEventDto {
            visible: should_present,
            displayed_wpm: (snapshot.live_metrics.displayed_wpm * 10.0).round() / 10.0,
            animation_band: snapshot.live_metrics.animation_band.as_str(),
            behavior: pip_behavior(
                snapshot.live_metrics.displayed_wpm,
                snapshot.live_metrics.active_typing_seconds,
            ),
            content: preferences.overlay_content.as_str(),
            size: preferences.overlay_size.as_str(),
            celebration_sequence: snapshot.live_metrics.celebration_sequence,
        };
        let tray_status = (
            snapshot.live_metrics.phase.overlay_visible(),
            snapshot.live_metrics.displayed_wpm.round().max(0.0) as u64,
        );
        if last_tray_status != Some(tray_status) {
            let icon_changed = last_tray_status.is_none_or(|previous| previous.0 != tray_status.0);
            if let Err(error) =
                crate::shell::update_brand_status(&app, tray_status.0, tray_status.1, icon_changed)
            {
                app.state::<DiagnosticState>()
                    .record_runtime_error(format!("menu-bar brand status failed: {error}"));
            }
            last_tray_status = Some(tray_status);
        }
        if last_payload.as_ref() != Some(&payload) {
            let _ = app.emit_to(OVERLAY_LABEL, OVERLAY_EVENT, payload.clone());
            last_payload = Some(payload);
        }

        if hide_after.is_some_and(|deadline| now >= deadline) {
            let _ = window.hide();
            presented = false;
            hide_after = None;
        }
        std::thread::sleep(UPDATE_INTERVAL);
    }

    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.hide();
    }
}

fn position_window<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    preferences: AppPreferences,
) -> tauri::Result<()> {
    let focused_monitor = typepulse_platform_macos::focused_window_center()
        .and_then(|point| app.monitor_from_point(point.x, point.y).ok().flatten());
    let current_monitor = window.current_monitor()?;
    let monitor = select_monitor(
        focused_monitor,
        current_monitor,
        app.primary_monitor()?,
        app.available_monitors()?.into_iter().next(),
    );
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let logical_size = dimensions(preferences.overlay_size);
    window.set_size(LogicalSize::new(logical_size.0, logical_size.1))?;
    let scale = monitor.scale_factor();
    let item_width = logical_to_physical(logical_size.0, scale);
    let item_height = logical_to_physical(logical_size.1, scale);
    let margin = logical_to_physical(SCREEN_MARGIN_LOGICAL, scale);
    let work_area = monitor.work_area();
    let position = corner_position(
        work_area.position.x,
        work_area.position.y,
        work_area.size.width,
        work_area.size.height,
        item_width,
        item_height,
        margin,
        preferences.overlay_position,
    );
    window.set_position(PhysicalPosition::new(position.0, position.1))
}

fn select_monitor<T>(
    focused: Option<T>,
    current: Option<T>,
    primary: Option<T>,
    available: Option<T>,
) -> Option<T> {
    focused.or(current).or(primary).or(available)
}

const fn dimensions(size: OverlaySize) -> (f64, f64) {
    match size {
        OverlaySize::Small => (168.0, 88.0),
        OverlaySize::Medium => (210.0, 108.0),
        OverlaySize::Large => (260.0, 132.0),
    }
}

fn logical_to_physical(value: f64, scale: f64) -> u32 {
    (value * scale).round().clamp(0.0, f64::from(u32::MAX)) as u32
}

fn pip_behavior(displayed_wpm: f64, active_typing_seconds: f64) -> &'static str {
    if active_typing_seconds.is_finite() && active_typing_seconds >= 90.0 * 60.0 {
        "tired"
    } else if !displayed_wpm.is_finite() || displayed_wpm <= 0.0 {
        "breathe"
    } else if displayed_wpm >= 70.0 {
        "run"
    } else {
        "walk"
    }
}

#[allow(clippy::too_many_arguments)]
fn corner_position(
    work_x: i32,
    work_y: i32,
    work_width: u32,
    work_height: u32,
    item_width: u32,
    item_height: u32,
    margin: u32,
    position: OverlayPosition,
) -> (i32, i32) {
    let right = matches!(
        position,
        OverlayPosition::TopRight | OverlayPosition::BottomRight
    );
    let bottom = matches!(
        position,
        OverlayPosition::BottomLeft | OverlayPosition::BottomRight
    );
    (
        axis_position(work_x, work_width, item_width, margin, right),
        axis_position(work_y, work_height, item_height, margin, bottom),
    )
}

fn axis_position(start: i32, span: u32, item: u32, margin: u32, trailing: bool) -> i32 {
    let offset = if trailing {
        span.saturating_sub(item.saturating_add(margin))
    } else {
        margin.min(span.saturating_sub(item))
    };
    i64::from(start)
        .saturating_add(i64::from(offset))
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use typepulse_core::{AppPreferences, OverlayContent, OverlayPosition, OverlaySize};

    use super::{
        corner_position, dimensions, pip_behavior, select_monitor, OverlayEventDto,
        OverlayPreferenceDto,
    };

    #[test]
    fn transient_focus_failure_keeps_the_current_monitor_before_primary_fallback() {
        assert_eq!(
            select_monitor(None, Some("current"), Some("primary"), Some("available")),
            Some("current")
        );
        assert_eq!(
            select_monitor(
                Some("focused"),
                Some("current"),
                Some("primary"),
                Some("available")
            ),
            Some("focused")
        );
    }

    #[test]
    fn positions_all_corners_inside_a_negative_origin_work_area() {
        let area = (-1920, 24, 1920, 1056, 210, 108, 20);
        assert_eq!(
            corner_position(
                area.0,
                area.1,
                area.2,
                area.3,
                area.4,
                area.5,
                area.6,
                OverlayPosition::TopLeft
            ),
            (-1900, 44)
        );
        assert_eq!(
            corner_position(
                area.0,
                area.1,
                area.2,
                area.3,
                area.4,
                area.5,
                area.6,
                OverlayPosition::TopRight
            ),
            (-230, 44)
        );
        assert_eq!(
            corner_position(
                area.0,
                area.1,
                area.2,
                area.3,
                area.4,
                area.5,
                area.6,
                OverlayPosition::BottomLeft
            ),
            (-1900, 952)
        );
        assert_eq!(
            corner_position(
                area.0,
                area.1,
                area.2,
                area.3,
                area.4,
                area.5,
                area.6,
                OverlayPosition::BottomRight
            ),
            (-230, 952)
        );
    }

    #[test]
    fn size_presets_are_strictly_increasing() {
        let small = dimensions(OverlaySize::Small);
        let medium = dimensions(OverlaySize::Medium);
        let large = dimensions(OverlaySize::Large);
        assert!(small.0 < medium.0 && medium.0 < large.0);
        assert!(small.1 < medium.1 && medium.1 < large.1);
    }

    #[test]
    fn brand_behaviors_have_measurable_boundaries_and_tired_precedence() {
        assert_eq!(pip_behavior(0.0, 0.0), "breathe");
        assert_eq!(pip_behavior(1.0, 0.0), "walk");
        assert_eq!(pip_behavior(69.999, 0.0), "walk");
        assert_eq!(pip_behavior(70.0, 0.0), "run");
        assert_eq!(pip_behavior(120.0, 5_399.999), "run");
        assert_eq!(pip_behavior(120.0, 5_400.0), "tired");
        assert_eq!(pip_behavior(f64::NAN, 0.0), "breathe");
    }

    #[test]
    fn preference_dto_contains_only_visual_configuration() {
        let value = serde_json::to_value(OverlayPreferenceDto::from(AppPreferences {
            auto_start_enabled: true,
            menu_bar_wpm_enabled: false,
            overlay_enabled: true,
            overlay_position: OverlayPosition::BottomRight,
            overlay_size: OverlaySize::Large,
            overlay_content: OverlayContent::Both,
        }))
        .unwrap();
        let Value::Object(object) = value else {
            panic!("overlay preference must serialize as an object");
        };
        assert_eq!(object.len(), 4);
        assert!(object.contains_key("enabled"));
        assert!(object.contains_key("position"));
        assert!(object.contains_key("size"));
        assert!(object.contains_key("content"));
    }

    #[test]
    fn overlay_event_dto_contains_only_aggregate_presentation_state() {
        let value = serde_json::to_value(OverlayEventDto {
            visible: true,
            displayed_wpm: 82.0,
            animation_band: "fast",
            behavior: "run",
            content: "both",
            size: "medium",
            celebration_sequence: 1,
        })
        .unwrap();
        let Value::Object(object) = value else {
            panic!("overlay event must serialize as an object");
        };
        assert_eq!(object.len(), 7);
        for expected in [
            "visible",
            "displayedWpm",
            "animationBand",
            "behavior",
            "content",
            "size",
            "celebrationSequence",
        ] {
            assert!(object.contains_key(expected));
        }
    }
}
