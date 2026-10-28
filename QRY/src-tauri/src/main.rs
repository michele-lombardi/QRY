//! Native executable entry point for QRY.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "macos")]
    if std::env::args().nth(1).as_deref() == Some("--configure-local-integrations") {
        // Explicit local installer operation: preferences only, without Tauri or input APIs.
        std::process::exit(if configure_local_integrations().is_ok() {
            0
        } else {
            1
        });
    }
    typepulse_lib::run()
}

#[cfg(target_os = "macos")]
fn configure_local_integrations() -> Result<(), Box<dyn std::error::Error>> {
    use typepulse_core::StatisticsRepository;
    let home = std::env::var_os("HOME").ok_or("home directory unavailable")?;
    let path = std::path::PathBuf::from(home)
        .join("Library/Application Support/app.typepulse.desktop/typepulse.sqlite3");
    let mut repository = typepulse_storage_sqlite::SqliteStatisticsRepository::open(path)?;
    let mut preferences = repository.load_preferences()?;
    preferences.capsule_enabled = true;
    preferences.integrations_enabled = true;
    preferences.notch_pip_enabled = true;
    repository.save_preferences(preferences)?;
    Ok(())
}
