//! TypePulse desktop application composition root.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the Tauri desktop runtime.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
