//! Persisted startup preference and OS login registration.

use crate::app_state::DiagnosticState;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use typepulse_platform_desktop::{input_permission_status, PermissionStatus};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegistrationAction {
    None,
    Enable,
    Disable,
}

const fn registration_action(desired: bool, registered: bool) -> RegistrationAction {
    match (desired, registered) {
        (true, false) => RegistrationAction::Enable,
        (false, true) => RegistrationAction::Disable,
        _ => RegistrationAction::None,
    }
}

const fn desired_registration(permission_gate_valid: bool, preference_enabled: bool) -> bool {
    permission_gate_valid && preference_enabled
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
    let preferences = state.load_preferences()?;
    if enabled
        && (!preferences.onboarding_completed
            || input_permission_status() != PermissionStatus::Granted)
    {
        return Err("complete required permission setup before enabling launch at login".into());
    }
    persist_auto_start_choice(&app, &state, preferences, enabled, false)?;
    startup_preference(app, state)
}

/// Applies the explicit onboarding choice only after required access is granted.
pub(crate) fn complete_onboarding_auto_start(
    app: &AppHandle,
    state: &DiagnosticState,
    enabled: bool,
) -> Result<(), String> {
    if input_permission_status() != PermissionStatus::Granted {
        return Err("Input Monitoring permission is still required".into());
    }
    let preferences = state.load_preferences()?;
    persist_auto_start_choice(app, state, preferences, enabled, true)
}

/// Reconciles the persisted preference with the real LaunchAgent state.
///
/// An invalid permission gate always removes the registration and clears the
/// preference so the database, checkbox and macOS state cannot drift apart.
pub(crate) fn reconcile_auto_start(
    app: &AppHandle,
    state: &DiagnosticState,
    permission_gate_valid: bool,
) -> Result<(), String> {
    let mut preferences = state.load_preferences()?;
    let desired = desired_registration(permission_gate_valid, preferences.auto_start_enabled);
    let previously_registered = registration_state(app)?;
    apply_registration(app, desired, previously_registered)?;

    if !permission_gate_valid && preferences.auto_start_enabled {
        preferences.auto_start_enabled = false;
        if let Err(error) = state.save_preferences(preferences) {
            restore_registration(app, previously_registered);
            return Err(error);
        }
    }
    Ok(())
}

fn persist_auto_start_choice(
    app: &AppHandle,
    state: &DiagnosticState,
    mut preferences: typepulse_core::AppPreferences,
    enabled: bool,
    complete_onboarding: bool,
) -> Result<(), String> {
    let previously_registered = registration_state(app)?;
    apply_registration(app, enabled, previously_registered)?;
    preferences.auto_start_enabled = enabled;
    if complete_onboarding {
        preferences.onboarding_completed = true;
    }
    if let Err(error) = state.save_preferences(preferences) {
        restore_registration(app, previously_registered);
        return Err(error);
    }
    Ok(())
}

fn registration_state(app: &AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

fn apply_registration(app: &AppHandle, desired: bool, registered: bool) -> Result<(), String> {
    match registration_action(desired, registered) {
        RegistrationAction::None => Ok(()),
        RegistrationAction::Enable => app.autolaunch().enable().map_err(|error| error.to_string()),
        RegistrationAction::Disable => app
            .autolaunch()
            .disable()
            .map_err(|error| error.to_string()),
    }
}

fn restore_registration(app: &AppHandle, registered: bool) {
    let current = registration_state(app).unwrap_or(!registered);
    let _ = apply_registration(app, registered, current);
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
    use super::{
        desired_registration, registration_action, MenuBarPreferenceDto, RegistrationAction,
    };

    #[test]
    fn menu_bar_preference_dto_contains_only_the_visibility_flag() {
        let dto = MenuBarPreferenceDto { wpm_enabled: true };
        assert!(dto.wpm_enabled);
    }

    #[test]
    fn reconciliation_is_idempotent_and_removes_stale_registrations() {
        assert_eq!(registration_action(false, false), RegistrationAction::None);
        assert_eq!(registration_action(true, true), RegistrationAction::None);
        assert_eq!(registration_action(true, false), RegistrationAction::Enable);
        assert_eq!(
            registration_action(false, true),
            RegistrationAction::Disable
        );
        assert!(desired_registration(true, true));
        assert!(!desired_registration(true, false));
        assert!(!desired_registration(false, true));
        assert!(!desired_registration(false, false));
    }
}
