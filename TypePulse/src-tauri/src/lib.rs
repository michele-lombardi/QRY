//! TypePulse desktop application composition root.

mod app_state;
mod commands;
mod overlay;
mod shell;

use app_state::DiagnosticState;
use commands::monitoring::{
    input_permission_status, monitor_status, open_input_settings, request_input_permission,
    start_input_monitoring, stop_input_monitoring,
};
use commands::preferences::{set_auto_start_enabled, startup_preference};
use commands::statistics::{
    export_daily_statistics_csv, recent_daily_summaries, reset_today_statistics, today_summary,
};
use overlay::{overlay_preference, set_overlay_preference};
use tauri::Manager;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use typepulse_storage_sqlite::SqliteStatisticsRepository;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the Tauri desktop runtime.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let database_path = app
                .path()
                .app_data_dir()
                .map_err(std::io::Error::other)?
                .join("typepulse.sqlite3");
            let repository =
                SqliteStatisticsRepository::open(database_path).map_err(std::io::Error::other)?;
            let state = DiagnosticState::new(repository);
            let preferences = state.load_preferences().map_err(std::io::Error::other)?;

            app.manage(state);
            overlay::configure(app, preferences)?;
            shell::configure(app)?;
            if preferences.auto_start_enabled {
                let state = app.state::<DiagnosticState>();
                if let Err(error) = app.autolaunch().enable() {
                    state.record_runtime_error(format!(
                        "automatic login registration failed: {error}"
                    ));
                }
                state.start_automatically();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    shell::hide_main_window(window);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            input_permission_status,
            request_input_permission,
            open_input_settings,
            monitor_status,
            start_input_monitoring,
            stop_input_monitoring,
            startup_preference,
            set_auto_start_enabled,
            overlay_preference,
            set_overlay_preference,
            today_summary,
            recent_daily_summaries,
            export_daily_statistics_csv,
            reset_today_statistics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
