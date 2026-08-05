//! Persisted startup preference and OS login registration.

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use typepulse_core::AppPreferences;

use crate::app_state::DiagnosticState;

/// Combined persisted preference and operating-system registration state.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartupPreferenceDto {
    auto_start_enabled: bool,
    login_item_registered: bool,
}

#[tauri::command]
pub(crate) fn startup_preference(
    app: AppHandle,
    state: State<'_, DiagnosticState>,
) -> Result<StartupPreferenceDto, String> {
    let preferences = state.load_preferences()?;
    let login_item_registered = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())?;
    Ok(StartupPreferenceDto {
        auto_start_enabled: preferences.auto_start_enabled,
        login_item_registered,
    })
}

#[tauri::command]
pub(crate) fn set_auto_start_enabled(
    enabled: bool,
    app: AppHandle,
    state: State<'_, DiagnosticState>,
) -> Result<StartupPreferenceDto, String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|error| error.to_string())?;
    } else {
        manager.disable().map_err(|error| error.to_string())?;
    }

    if let Err(error) = state.save_preferences(AppPreferences {
        auto_start_enabled: enabled,
    }) {
        if enabled {
            let _ = manager.disable();
        } else {
            let _ = manager.enable();
        }
        return Err(error);
    }

    if enabled && state.snapshot().run_state == typepulse_platform_macos::MonitorRunState::Stopped {
        state.start_automatically();
    }
    startup_preference(app, state)
}
