//! TypePulse desktop application composition root.

mod app_state;
mod commands;

use app_state::DiagnosticState;
use commands::monitoring::{
    input_permission_status, monitor_status, open_input_settings, request_input_permission,
    start_input_monitoring, stop_input_monitoring,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the Tauri desktop runtime.
pub fn run() {
    tauri::Builder::default()
        .manage(DiagnosticState::default())
        .invoke_handler(tauri::generate_handler![
            input_permission_status,
            request_input_permission,
            open_input_settings,
            monitor_status,
            start_input_monitoring,
            stop_input_monitoring,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
