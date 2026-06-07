//! Composition root: wires app state and the command surface into Tauri.

mod commands;
mod error;
mod protocol;
mod state;
mod transport;

#[cfg(test)]
mod wire_e2e;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::scan_devices,
            commands::connect_device,
            commands::disconnect_device,
            commands::send_request,
            commands::connection_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
