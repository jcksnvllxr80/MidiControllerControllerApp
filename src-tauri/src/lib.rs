//! Composition root: wires app state and the command surface into Tauri.

mod commands;
mod config;
mod error;
mod firmware;
mod logging;
mod protocol;
mod state;
mod transport;

#[cfg(test)]
mod wire_e2e;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Resolve the log directory: prefer an explicit config.toml override, then
    // fall back to a `logs` folder next to the executable.
    let cfg = config::AppConfig::load();
    let log_dir = cfg.log_path.unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join("logs")))
            .unwrap_or_else(|| std::path::PathBuf::from("logs"))
    });

    if let Err(e) = logging::init(&log_dir) {
        eprintln!("warning: could not initialise logging to {}: {e}", log_dir.display());
    } else {
        log::info!("===== MidiController app session started =====");
        log::info!("logs directory: {}", log_dir.display());
        // Write a session-start sentinel to device.log so it's easy to find
        // where each run begins when scrolling through the file.
        log::info!(target: "device", "===== device log session started =====");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new().with_log_dir(log_dir))
        .invoke_handler(tauri::generate_handler![
            commands::scan_devices,
            commands::connect_device,
            commands::disconnect_device,
            commands::send_request,
            commands::connection_status,
            commands::find_bootloader,
            commands::flash_firmware,
            commands::get_log_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
