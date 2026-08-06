//! QRY desktop application composition root.

mod app_state;
mod commands;
mod overlay;
mod permission_flow;
mod shell;

use app_state::DiagnosticState;
use commands::monitoring::{
    accessibility_permission_status, input_permission_status, monitor_status,
    open_accessibility_permission_settings, open_input_settings, request_accessibility_permission,
    request_input_permission, start_input_monitoring, stop_input_monitoring,
};
use commands::preferences::{
    menu_bar_preference, reconcile_auto_start, set_auto_start_enabled, set_menu_bar_wpm_enabled,
    startup_preference,
};
use commands::statistics::{
    export_daily_statistics_csv, recent_daily_summaries, reset_today_statistics,
    today_metric_buckets, today_summary,
};
use overlay::{overlay_preference, set_overlay_preference};
use permission_flow::{
    begin_permission_flow, complete_permission_flow, exit_permission_flow, permission_flow_status,
    wait_for_input_permission,
};
use shell::{
    hide_dashboard_window, open_settings_window, open_statistics_window, open_today_window,
};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use typepulse_storage_sqlite::SqliteStatisticsRepository;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the Tauri desktop runtime.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            permission_flow::focus_primary_surface(app);
        }))
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
            shell::configure(app, preferences)?;
            overlay::configure(app, preferences)?;
            let permission_runtime =
                permission_flow::configure(app, preferences.onboarding_completed);
            let normal_start = permission_runtime.permits_normal_start();
            if normal_start {
                let state = app.state::<DiagnosticState>();
                match state.start() {
                    Ok(()) => {
                        if let Err(error) = reconcile_auto_start(app.handle(), &state, true) {
                            state.record_runtime_error(format!(
                                "automatic login reconciliation failed: {error}"
                            ));
                        }
                        permission_flow::start_revocation_watchdog(
                            app.handle().clone(),
                            permission_runtime,
                        );
                    }
                    Err(error) => {
                        state.record_runtime_error(error);
                        permission_runtime.mark_monitor_failed();
                        if let Err(error) = reconcile_auto_start(app.handle(), &state, false) {
                            state.record_runtime_error(format!(
                                "automatic login cleanup after monitor failure failed: {error}"
                            ));
                        }
                        permission_flow::show_gate(app.handle());
                    }
                }
            } else {
                let state = app.state::<DiagnosticState>();
                if let Err(error) = reconcile_auto_start(app.handle(), &state, false) {
                    state.record_runtime_error(format!(
                        "automatic login cleanup before permission setup failed: {error}"
                    ));
                }
                permission_flow::show_gate(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == permission_flow::GATE_LABEL {
                    api.prevent_close();
                    permission_flow::exit_from_gate(window.app_handle());
                } else if matches!(window.label(), "main" | "statistics" | "dashboard") {
                    api.prevent_close();
                    shell::hide_main_window(window);
                }
            }
            if matches!(event, tauri::WindowEvent::Focused(false)) && window.label() == "dashboard"
            {
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            input_permission_status,
            request_input_permission,
            open_input_settings,
            accessibility_permission_status,
            request_accessibility_permission,
            open_accessibility_permission_settings,
            monitor_status,
            start_input_monitoring,
            stop_input_monitoring,
            startup_preference,
            set_auto_start_enabled,
            menu_bar_preference,
            set_menu_bar_wpm_enabled,
            overlay_preference,
            set_overlay_preference,
            today_summary,
            recent_daily_summaries,
            today_metric_buckets,
            export_daily_statistics_csv,
            reset_today_statistics,
            open_settings_window,
            open_statistics_window,
            open_today_window,
            hide_dashboard_window,
            permission_flow_status,
            begin_permission_flow,
            wait_for_input_permission,
            complete_permission_flow,
            exit_permission_flow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
