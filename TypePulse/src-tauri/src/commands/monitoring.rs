//! Diagnostic commands for the Phase B macOS monitor spike.

use serde::Serialize;
use tauri::State;
use typepulse_platform_macos::{
    input_permission_status as platform_permission_status, open_input_monitoring_settings,
    request_input_permission as platform_request_permission, PermissionStatus,
};

use crate::app_state::DiagnosticState;

/// Serializable permission response with no input metadata.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PermissionStatusDto {
    status: &'static str,
}

impl From<PermissionStatus> for PermissionStatusDto {
    fn from(status: PermissionStatus) -> Self {
        Self {
            status: status.as_str(),
        }
    }
}

/// Serializable monitor health snapshot.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MonitorStatusDto {
    state: &'static str,
    total_activities: u64,
    events_seen: u64,
    activities_emitted: u64,
    activities_dropped: u64,
    callback_count: u64,
    average_callback_ns: u64,
    max_callback_ns: u64,
    reenable_attempts: u64,
    session_phase: &'static str,
    raw_wpm: f64,
    displayed_wpm: f64,
    animation_band: &'static str,
    last_error: Option<String>,
}

#[tauri::command]
pub(crate) fn input_permission_status() -> PermissionStatusDto {
    platform_permission_status().into()
}

#[tauri::command]
pub(crate) fn request_input_permission() -> PermissionStatusDto {
    platform_request_permission().into()
}

#[tauri::command]
pub(crate) fn open_input_settings() -> Result<(), String> {
    open_input_monitoring_settings().map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn monitor_status(state: State<'_, DiagnosticState>) -> MonitorStatusDto {
    let snapshot = state.snapshot();
    MonitorStatusDto {
        state: snapshot.run_state.as_str(),
        total_activities: snapshot.total_activities,
        events_seen: snapshot.monitor_metrics.events_seen,
        activities_emitted: snapshot.monitor_metrics.activities_emitted,
        activities_dropped: snapshot.monitor_metrics.activities_dropped,
        callback_count: snapshot.monitor_metrics.callback_count,
        average_callback_ns: snapshot.monitor_metrics.average_callback_ns,
        max_callback_ns: snapshot.monitor_metrics.max_callback_ns,
        reenable_attempts: snapshot.monitor_metrics.reenable_attempts,
        session_phase: snapshot.live_metrics.phase.as_str(),
        raw_wpm: snapshot.live_metrics.raw_wpm,
        displayed_wpm: snapshot.live_metrics.displayed_wpm,
        animation_band: snapshot.live_metrics.animation_band.as_str(),
        last_error: snapshot.last_error,
    }
}

#[tauri::command]
pub(crate) fn start_input_monitoring(
    state: State<'_, DiagnosticState>,
) -> Result<MonitorStatusDto, String> {
    state.start()?;
    Ok(monitor_status(state))
}

#[tauri::command]
pub(crate) fn stop_input_monitoring(
    state: State<'_, DiagnosticState>,
) -> Result<MonitorStatusDto, String> {
    state.stop()?;
    Ok(monitor_status(state))
}

#[cfg(test)]
mod tests {
    use typepulse_platform_macos::PermissionStatus;

    use super::PermissionStatusDto;

    #[test]
    fn permission_dto_contains_only_the_status() {
        let dto = PermissionStatusDto::from(PermissionStatus::Denied);
        assert_eq!(dto.status, "denied");
    }
}
