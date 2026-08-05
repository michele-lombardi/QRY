//! macOS menu-bar shell and background window lifecycle.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tauri::{
    image::Image,
    menu::{CheckMenuItem, MenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};
use typepulse_core::AppPreferences;

use crate::app_state::DiagnosticState;

const TRAY_ID: &str = "typepulse-menu-bar";
const OPEN_ID: &str = "open";
const SHOW_WPM_ID: &str = "show-wpm";
const START_ID: &str = "start";
const PAUSE_ID: &str = "pause";
const QUIT_ID: &str = "quit";
const WPM_SLOT_WIDTH: usize = 3;
const FIGURE_SPACE: char = '\u{2007}';

/// Latest menu-bar presentation state, shared without polling preferences.
pub(crate) struct TrayRuntime {
    show_wpm: AtomicBool,
    active: AtomicBool,
    displayed_wpm: AtomicU64,
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

/// Configures TypePulse as a background menu-bar application.
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
        .text(OPEN_ID, "Open TypePulse")
        .separator()
        .item(&show_wpm_item)
        .separator()
        .text(START_ID, "Start monitoring")
        .text(PAUSE_ID, "Pause monitoring")
        .separator()
        .text(QUIT_ID, "Quit TypePulse")
        .build()?;

    app.manage(TrayRuntime::new(preferences.menu_bar_wpm_enabled));
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("TypePulse")
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
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
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
    if window.label() == "main" {
        let _ = window.hide();
    }
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn handle_menu_action<R: Runtime>(
    app: &AppHandle<R>,
    action: MenuAction,
    show_wpm_item: &CheckMenuItem<R>,
) {
    match action {
        MenuAction::Open => show_main_window(app),
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
    let state = app.state::<DiagnosticState>();
    let result = state.load_preferences().and_then(|mut preferences| {
        preferences.menu_bar_wpm_enabled = enabled;
        state.save_preferences(preferences)
    });

    if let Err(error) = result {
        let _ = item.set_checked(previous);
        state.record_runtime_error(format!("menu-bar WPM preference failed: {error}"));
        return;
    }

    runtime.show_wpm.store(enabled, Ordering::Release);
    let _ = item.set_checked(enabled);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Err(error) = apply_menu_bar_title(&tray, &runtime) {
            state.record_runtime_error(format!("menu-bar WPM visibility failed: {error}"));
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Open,
    ToggleWpm,
    Start,
    Pause,
    Quit,
    Unknown,
}

impl MenuAction {
    fn from_id(id: &str) -> Self {
        match id {
            OPEN_ID => Self::Open,
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

    use super::{menu_bar_title, MenuAction, FIGURE_SPACE};

    #[test]
    fn tray_menu_ids_map_only_to_known_actions() {
        assert_eq!(MenuAction::from_id("open"), MenuAction::Open);
        assert_eq!(MenuAction::from_id("show-wpm"), MenuAction::ToggleWpm);
        assert_eq!(MenuAction::from_id("start"), MenuAction::Start);
        assert_eq!(MenuAction::from_id("pause"), MenuAction::Pause);
        assert_eq!(MenuAction::from_id("quit"), MenuAction::Quit);
        assert_eq!(MenuAction::from_id("unexpected"), MenuAction::Unknown);
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
