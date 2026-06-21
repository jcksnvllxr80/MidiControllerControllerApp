//! Tauri command surface exposed to the Svelte frontend. Thin wrappers around
//! `AppState` (which holds the testable logic), plus event emission. Each runs
//! as an async task so blocking transport I/O stays off the main thread.

use tauri::{AppHandle, Emitter, State};

use crate::error::AppError;
use crate::protocol::{Request, Response};
use crate::state::{AppState, ConnectionStatus};
use crate::transport::DeviceInfo;

/// Scan every transport for candidate devices. Emits `device-found` per device
/// (so the UI can render progressively) and also returns the full list.
#[tauri::command]
pub async fn scan_devices(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<DeviceInfo>, AppError> {
    log::info!("scanning for devices");
    let devices = state.registry.discover_all();
    log::info!("scan found {} device(s)", devices.len());
    for d in &devices {
        let _ = app.emit("device-found", d.clone());
    }
    Ok(devices)
}

#[tauri::command]
pub async fn connect_device(
    app: AppHandle,
    state: State<'_, AppState>,
    device: DeviceInfo,
) -> Result<ConnectionStatus, AppError> {
    log::info!("connecting to '{}' ({})", device.name, device.id);
    let status = state.connect(device).map_err(|e| {
        log::warn!("connection failed: {e}");
        e
    })?;
    if let Some(id) = &status.identity {
        log::info!("connected: {} firmware {}", id.name, id.firmware);
    }
    let _ = app.emit("connection-status", status.clone());
    for line in state.drain_logs() {
        let _ = app.emit("device-log", line);
    }
    Ok(status)
}

#[tauri::command]
pub async fn disconnect_device(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ConnectionStatus, AppError> {
    log::info!("disconnecting from device");
    let status = state.disconnect()?;
    let _ = app.emit("connection-status", status.clone());
    Ok(status)
}

#[tauri::command]
pub async fn send_request(
    app: AppHandle,
    state: State<'_, AppState>,
    request: Request,
) -> Result<Response, AppError> {
    // Drain logs even on error so firmware output isn't silently swallowed when
    // a request times out (the logs often explain *why* it failed).
    let result = state.send(request);
    if let Err(ref e) = result {
        log::warn!("request error: {e}");
    }
    for line in state.drain_logs() {
        let _ = app.emit("device-log", line);
    }
    result
}

#[tauri::command]
pub async fn connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, AppError> {
    Ok(state.status())
}

/// Return the directory where `app.log` and `device.log` are written.
/// Empty string if logging could not be initialised at startup.
#[tauri::command]
pub async fn get_log_dir(state: State<'_, AppState>) -> Result<String, AppError> {
    Ok(state.get_log_dir())
}

/// Look for the RP2350 UF2 bootloader drive (present after `reboot_bootloader`).
#[tauri::command]
pub async fn find_bootloader() -> Option<crate::firmware::BootloaderDrive> {
    crate::firmware::find_bootloader()
}

/// Flash a `.uf2` by copying it onto the bootloader drive. Requires the device
/// to be in bootloader mode (USB), regardless of how it was connected.
#[tauri::command]
pub async fn flash_firmware(uf2_path: String) -> Result<String, AppError> {
    log::info!("flashing firmware from {uf2_path}");
    let drive = crate::firmware::find_bootloader().ok_or_else(|| {
        AppError::Internal(
            "No RP2350 bootloader drive found — put the device in bootloader mode (Update firmware) and plug in USB."
                .into(),
        )
    })?;
    let result = crate::firmware::flash_uf2(&uf2_path, &drive.mount_point)
        .map_err(AppError::Internal);
    match &result {
        Ok(dest) => log::info!("firmware flashed to {dest}"),
        Err(e) => log::warn!("firmware flash failed: {e}"),
    }
    result
}
