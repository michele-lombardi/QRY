//! Persisted startup preference and OS login registration.

use crate::app_state::DiagnosticState;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

/// Combined persisted preference and operating-system registration state.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartupPreferenceDto {
    auto_start_enabled: bool,
    login_item_registered: bool,
}

/// Persisted menu-bar presentation preference.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MenuBarPreferenceDto {
    wpm_enabled: bool,
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

    let mut preferences = state.load_preferences()?;
    preferences.auto_start_enabled = enabled;
    if let Err(error) = state.save_preferences(preferences) {
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

#[tauri::command]
pub(crate) fn menu_bar_preference(
    state: State<'_, DiagnosticState>,
) -> Result<MenuBarPreferenceDto, String> {
    Ok(MenuBarPreferenceDto {
        wpm_enabled: state.load_preferences()?.menu_bar_wpm_enabled,
    })
}

#[tauri::command]
pub(crate) fn set_menu_bar_wpm_enabled(
    enabled: bool,
    app: AppHandle,
) -> Result<MenuBarPreferenceDto, String> {
    crate::shell::set_menu_bar_wpm_enabled(&app, enabled)?;
    Ok(MenuBarPreferenceDto {
        wpm_enabled: enabled,
    })
}

#[cfg(test)]
mod tests {
    use super::MenuBarPreferenceDto;

    #[test]
    fn menu_bar_preference_dto_contains_only_the_visibility_flag() {
        let dto = MenuBarPreferenceDto { wpm_enabled: true };
        assert!(dto.wpm_enabled);
    }
}
