//! macOS menu-bar shell and background window lifecycle.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tauri::{
    image::Image,
    menu::{CheckMenuItem, MenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, LogicalPosition, Manager, PhysicalPosition, Runtime,
};
use typepulse_core::AppPreferences;

use crate::app_state::DiagnosticState;

const TRAY_ID: &str = "typepulse-menu-bar";
const TODAY_ID: &str = "today";
const SETTINGS_ID: &str = "settings";
const STATISTICS_ID: &str = "statistics";
const SHOW_WPM_ID: &str = "show-wpm";
const START_ID: &str = "start";
const PAUSE_ID: &str = "pause";
const QUIT_ID: &str = "quit";
const WPM_SLOT_WIDTH: usize = 3;
const FIGURE_SPACE: char = '\u{2007}';
const DASHBOARD_LABEL: &str = "dashboard";
const SETTINGS_LABEL: &str = "main";
const STATISTICS_LABEL: &str = "statistics";
const DASHBOARD_WIDTH_LOGICAL: f64 = 360.0;
const DASHBOARD_MARGIN_LOGICAL: f64 = 10.0;

/// Latest menu-bar presentation state, shared without polling preferences.
pub(crate) struct TrayRuntime {
    show_wpm: AtomicBool,
    active: AtomicBool,
    displayed_wpm: AtomicU64,
}

struct TrayMenuItems {
    show_wpm: CheckMenuItem<tauri::Wry>,
}

impl TrayRuntime {
    fn new(show_wpm: bool) -> Self {
        Self {
            show_wpm: AtomicBool::new(show_wpm),
            active: AtomicBool::new(false),
            displayed_wpm: AtomicU64::new(0),
        }
    }
}

/// Configures QRY as a background menu-bar application.
pub(crate) fn configure(app: &mut App, preferences: AppPreferences) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let show_wpm_item = CheckMenuItem::with_id(
        app,
        SHOW_WPM_ID,
        "Show WPM in menu bar",
        true,
        preferences.menu_bar_wpm_enabled,
        None::<&str>,
    )?;
    let menu = MenuBuilder::new(app)
        .text(TODAY_ID, "Today")
        .text(STATISTICS_ID, "Statistics…")
        .text(SETTINGS_ID, "Settings…")
        .separator()
        .item(&show_wpm_item)
        .separator()
        .text(START_ID, "Start monitoring")
        .text(PAUSE_ID, "Pause monitoring")
        .separator()
        .text(QUIT_ID, "Quit QRY")
        .build()?;

    app.manage(TrayRuntime::new(preferences.menu_bar_wpm_enabled));
    app.manage(TrayMenuItems {
        show_wpm: show_wpm_item.clone(),
    });
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("QRY")
        .icon(Image::from_bytes(include_bytes!("../icons/tray-idle.png"))?);
    if let Some(title) = menu_bar_title(preferences.menu_bar_wpm_enabled, false, 0) {
        tray = tray.title(title);
    }
    let event_show_wpm_item = show_wpm_item.clone();
    let tray = tray
        .on_menu_event(move |app, event| {
            handle_menu_action(
                app,
                MenuAction::from_id(event.id().as_ref()),
                &event_show_wpm_item,
            );
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                position,
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_dashboard(tray.app_handle(), position);
            }
        });
    tray.build(app)?;
    Ok(())
}

/// Applies the compact Pulse mark and live WPM title to the menu-bar item.
pub(crate) fn update_brand_status<R: Runtime>(
    app: &AppHandle<R>,
    active: bool,
    displayed_wpm: u64,
    icon_changed: bool,
) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let runtime = app.state::<TrayRuntime>();
    runtime.active.store(active, Ordering::Release);
    runtime
        .displayed_wpm
        .store(displayed_wpm, Ordering::Release);
    if icon_changed {
        let bytes = if active {
            include_bytes!("../icons/tray-active.png").as_slice()
        } else {
            include_bytes!("../icons/tray-idle.png").as_slice()
        };
        tray.set_icon_with_as_template(Some(Image::from_bytes(bytes)?), cfg!(target_os = "macos"))?;
    }
    apply_menu_bar_title(&tray, &runtime)
}

fn apply_menu_bar_title<R: Runtime>(
    tray: &TrayIcon<R>,
    runtime: &TrayRuntime,
) -> tauri::Result<()> {
    tray.set_title(menu_bar_title(
        runtime.show_wpm.load(Ordering::Acquire),
        runtime.active.load(Ordering::Acquire),
        runtime.displayed_wpm.load(Ordering::Acquire),
    ))
}

fn menu_bar_title(enabled: bool, active: bool, displayed_wpm: u64) -> Option<String> {
    if !enabled {
        return None;
    }
    let value = if active {
        displayed_wpm.min(999).to_string()
    } else {
        String::new()
    };
    let padding = WPM_SLOT_WIDTH.saturating_sub(value.chars().count());
    Some(
        std::iter::repeat_n(FIGURE_SPACE, padding)
            .chain(value.chars())
            .collect(),
    )
}

/// Hides the main window instead of terminating the background process.
pub(crate) fn hide_main_window<R: Runtime>(window: &tauri::Window<R>) {
    if matches!(
        window.label(),
        SETTINGS_LABEL | STATISTICS_LABEL | DASHBOARD_LABEL
    ) {
        let _ = window.hide();
    }
}

fn show_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if let Some(dashboard) = app.get_webview_window(DASHBOARD_LABEL) {
        if label != DASHBOARD_LABEL {
            let _ = dashboard.hide();
        }
    }
    for full_label in [SETTINGS_LABEL, STATISTICS_LABEL] {
        if full_label != label {
            if let Some(window) = app.get_webview_window(full_label) {
                let _ = window.hide();
            }
        }
    }
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_dashboard<R: Runtime>(app: &AppHandle<R>, tray_position: PhysicalPosition<f64>) {
    let Some(window) = app.get_webview_window(DASHBOARD_LABEL) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    position_dashboard(app, Some(tray_position));
    show_window(app, DASHBOARD_LABEL);
}

fn position_dashboard<R: Runtime>(
    app: &AppHandle<R>,
    tray_position: Option<PhysicalPosition<f64>>,
) {
    let Some(window) = app.get_webview_window(DASHBOARD_LABEL) else {
        return;
    };
    let monitor = tray_position
        .and_then(|position| {
            app.monitor_from_point(position.x, position.y)
                .ok()
                .flatten()
        })
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let work = monitor.work_area();
        let panel_width = (DASHBOARD_WIDTH_LOGICAL * scale).round() as u32;
        let margin = (DASHBOARD_MARGIN_LOGICAL * scale).round() as u32;
        let anchor = dashboard_anchor(
            work.position.x,
            work.position.y,
            work.size.width,
            panel_width,
            margin,
        );
        let _ = window.set_position(LogicalPosition::new(
            f64::from(anchor.0) / scale,
            f64::from(anchor.1) / scale,
        ));
    }
}

fn dashboard_anchor(
    work_x: i32,
    work_y: i32,
    work_width: u32,
    panel_width: u32,
    margin: u32,
) -> (i32, i32) {
    let offset = work_width.saturating_sub(panel_width.saturating_add(margin));
    (
        i64::from(work_x)
            .saturating_add(i64::from(offset))
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        i64::from(work_y)
            .saturating_add(i64::from(margin))
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    )
}

#[tauri::command]
pub(crate) fn open_settings_window(app: AppHandle) {
    show_window(&app, SETTINGS_LABEL);
}

#[tauri::command]
pub(crate) fn open_today_window(app: AppHandle) {
    position_dashboard(&app, None);
    show_window(&app, DASHBOARD_LABEL);
}

#[tauri::command]
pub(crate) fn open_statistics_window(app: AppHandle) {
    show_window(&app, STATISTICS_LABEL);
}

#[tauri::command]
pub(crate) fn hide_dashboard_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window(DASHBOARD_LABEL) {
        let _ = window.hide();
    }
}

fn handle_menu_action<R: Runtime>(
    app: &AppHandle<R>,
    action: MenuAction,
    show_wpm_item: &CheckMenuItem<R>,
) {
    match action {
        MenuAction::Today => {
            position_dashboard(app, None);
            show_window(app, DASHBOARD_LABEL);
        }
        MenuAction::Settings => show_window(app, SETTINGS_LABEL),
        MenuAction::Statistics => show_window(app, STATISTICS_LABEL),
        MenuAction::ToggleWpm => toggle_menu_bar_wpm(app, show_wpm_item),
        MenuAction::Start => app.state::<DiagnosticState>().start_automatically(),
        MenuAction::Pause => {
            let state = app.state::<DiagnosticState>();
            if let Err(error) = state.stop() {
                state.record_runtime_error(error);
            }
        }
        MenuAction::Quit => {
            app.state::<crate::overlay::OverlayRuntime>().stop();
            let state = app.state::<DiagnosticState>();
            if let Err(error) = state.stop() {
                state.record_runtime_error(error);
            }
            app.exit(0);
        }
        MenuAction::Unknown => {}
    }
}

fn toggle_menu_bar_wpm<R: Runtime>(app: &AppHandle<R>, item: &CheckMenuItem<R>) {
    let runtime = app.state::<TrayRuntime>();
    let previous = runtime.show_wpm.load(Ordering::Acquire);
    let enabled = !previous;
    if let Err(error) = apply_menu_bar_wpm(app, item, enabled) {
        let _ = item.set_checked(previous);
        app.state::<DiagnosticState>()
            .record_runtime_error(format!("menu-bar WPM preference failed: {error}"));
    }
}

fn apply_menu_bar_wpm<R: Runtime>(
    app: &AppHandle<R>,
    item: &CheckMenuItem<R>,
    enabled: bool,
) -> Result<(), String> {
    let runtime = app.state::<TrayRuntime>();
    let previous = runtime.show_wpm.load(Ordering::Acquire);
    let state = app.state::<DiagnosticState>();
    let mut preferences = state.load_preferences()?;
    preferences.menu_bar_wpm_enabled = enabled;
    state.save_preferences(preferences)?;

    runtime.show_wpm.store(enabled, Ordering::Release);
    let presentation_result = item.set_checked(enabled).and_then(|()| {
        app.tray_by_id(TRAY_ID)
            .map_or(Ok(()), |tray| apply_menu_bar_title(&tray, &runtime))
    });
    if let Err(error) = presentation_result {
        runtime.show_wpm.store(previous, Ordering::Release);
        preferences.menu_bar_wpm_enabled = previous;
        let _ = state.save_preferences(preferences);
        let _ = item.set_checked(previous);
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = apply_menu_bar_title(&tray, &runtime);
        }
        return Err(error.to_string());
    }
    Ok(())
}

pub(crate) fn set_menu_bar_wpm_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let item = app.state::<TrayMenuItems>().show_wpm.clone();
    apply_menu_bar_wpm(app, &item, enabled)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Today,
    Settings,
    Statistics,
    ToggleWpm,
    Start,
    Pause,
    Quit,
    Unknown,
}

impl MenuAction {
    fn from_id(id: &str) -> Self {
        match id {
            TODAY_ID => Self::Today,
            SETTINGS_ID => Self::Settings,
            STATISTICS_ID => Self::Statistics,
            SHOW_WPM_ID => Self::ToggleWpm,
            START_ID => Self::Start,
            PAUSE_ID => Self::Pause,
            QUIT_ID => Self::Quit,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use tauri::image::Image;

    use super::{dashboard_anchor, menu_bar_title, MenuAction, FIGURE_SPACE};

    #[test]
    fn tray_menu_ids_map_only_to_known_actions() {
        assert_eq!(MenuAction::from_id("today"), MenuAction::Today);
        assert_eq!(MenuAction::from_id("settings"), MenuAction::Settings);
        assert_eq!(MenuAction::from_id("statistics"), MenuAction::Statistics);
        assert_eq!(MenuAction::from_id("show-wpm"), MenuAction::ToggleWpm);
        assert_eq!(MenuAction::from_id("start"), MenuAction::Start);
        assert_eq!(MenuAction::from_id("pause"), MenuAction::Pause);
        assert_eq!(MenuAction::from_id("quit"), MenuAction::Quit);
        assert_eq!(MenuAction::from_id("unexpected"), MenuAction::Unknown);
    }

    #[test]
    fn dashboard_is_anchored_to_the_top_right_of_negative_work_areas() {
        assert_eq!(dashboard_anchor(-3_840, -98, 1_920, 360, 10), (-2_290, -88));
    }

    #[test]
    fn wpm_title_uses_a_stable_three_digit_slot_or_disappears() {
        let blank = FIGURE_SPACE.to_string().repeat(3);
        assert_eq!(menu_bar_title(true, false, 0), Some(blank));
        assert_eq!(
            menu_bar_title(true, true, 9),
            Some(format!("{FIGURE_SPACE}{FIGURE_SPACE}9"))
        );
        assert_eq!(
            menu_bar_title(true, true, 42),
            Some(format!("{FIGURE_SPACE}42"))
        );
        assert_eq!(menu_bar_title(true, true, 120), Some("120".into()));
        assert_eq!(menu_bar_title(true, true, 1_200), Some("999".into()));
        assert_eq!(menu_bar_title(false, true, 80), None);
    }

    #[test]
    fn brand_tray_assets_are_valid_monochrome_three_to_two_pngs() {
        for bytes in [
            include_bytes!("../icons/tray-active.png").as_slice(),
            include_bytes!("../icons/tray-idle.png").as_slice(),
        ] {
            let image = Image::from_bytes(bytes).unwrap();
            assert_eq!((image.width(), image.height()), (48, 32));
            assert!(image.rgba().chunks_exact(4).any(|pixel| pixel[3] > 0));
            for pixel in image.rgba().chunks_exact(4).filter(|pixel| pixel[3] > 0) {
                assert_eq!(pixel[0], pixel[1]);
                assert_eq!(pixel[1], pixel[2]);
            }
        }
    }
}
