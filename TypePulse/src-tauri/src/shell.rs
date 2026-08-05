//! macOS menu-bar shell and background window lifecycle.

use tauri::{
    image::Image,
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

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("TypePulse")
        .icon(Image::from_bytes(include_bytes!("../icons/tray-idle.png"))?)
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
    if icon_changed {
        let bytes = if active {
            include_bytes!("../icons/tray-active.png").as_slice()
        } else {
            include_bytes!("../icons/tray-idle.png").as_slice()
        };
        tray.set_icon(Some(Image::from_bytes(bytes)?))?;
    }
    tray.set_title(active.then(|| displayed_wpm.to_string()))
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
    use tauri::image::Image;

    use super::MenuAction;

    #[test]
    fn tray_menu_ids_map_only_to_known_actions() {
        assert_eq!(MenuAction::from_id("open"), MenuAction::Open);
        assert_eq!(MenuAction::from_id("start"), MenuAction::Start);
        assert_eq!(MenuAction::from_id("pause"), MenuAction::Pause);
        assert_eq!(MenuAction::from_id("quit"), MenuAction::Quit);
        assert_eq!(MenuAction::from_id("unexpected"), MenuAction::Unknown);
    }

    #[test]
    fn brand_tray_assets_are_valid_three_to_two_pngs() {
        for bytes in [
            include_bytes!("../icons/tray-active.png").as_slice(),
            include_bytes!("../icons/tray-idle.png").as_slice(),
        ] {
            let image = Image::from_bytes(bytes).unwrap();
            assert_eq!((image.width(), image.height()), (48, 32));
        }
    }
}
