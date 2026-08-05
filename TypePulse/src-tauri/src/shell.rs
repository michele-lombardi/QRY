//! macOS menu-bar shell and background window lifecycle.

use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};

use crate::app_state::DiagnosticState;

const TRAY_ID: &str = "typepulse-menu-bar";
const OPEN_ID: &str = "open";
const START_ID: &str = "start";
const PAUSE_ID: &str = "pause";
const QUIT_ID: &str = "quit";

/// Configures TypePulse as a background menu-bar application.
pub(crate) fn configure(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let menu = MenuBuilder::new(app)
        .text(OPEN_ID, "Open TypePulse")
        .separator()
        .text(START_ID, "Start monitoring")
        .text(PAUSE_ID, "Pause monitoring")
        .separator()
        .text(QUIT_ID, "Quit TypePulse")
        .build()?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("TypePulse")
        .on_menu_event(|app, event| {
            handle_menu_action(app, MenuAction::from_id(event.id().as_ref()))
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
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
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

fn handle_menu_action<R: Runtime>(app: &AppHandle<R>, action: MenuAction) {
    match action {
        MenuAction::Open => show_main_window(app),
        MenuAction::Start => app.state::<DiagnosticState>().start_automatically(),
        MenuAction::Pause => {
            let state = app.state::<DiagnosticState>();
            if let Err(error) = state.stop() {
                state.record_runtime_error(error);
            }
        }
        MenuAction::Quit => {
            let state = app.state::<DiagnosticState>();
            if let Err(error) = state.stop() {
                state.record_runtime_error(error);
            }
            app.exit(0);
        }
        MenuAction::Unknown => {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Open,
    Start,
    Pause,
    Quit,
    Unknown,
}

impl MenuAction {
    fn from_id(id: &str) -> Self {
        match id {
            OPEN_ID => Self::Open,
            START_ID => Self::Start,
            PAUSE_ID => Self::Pause,
            QUIT_ID => Self::Quit,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MenuAction;

    #[test]
    fn tray_menu_ids_map_only_to_known_actions() {
        assert_eq!(MenuAction::from_id("open"), MenuAction::Open);
        assert_eq!(MenuAction::from_id("start"), MenuAction::Start);
        assert_eq!(MenuAction::from_id("pause"), MenuAction::Pause);
        assert_eq!(MenuAction::from_id("quit"), MenuAction::Quit);
        assert_eq!(MenuAction::from_id("unexpected"), MenuAction::Unknown);
    }
}
