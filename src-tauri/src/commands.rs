//! Tauri command surface exposed to the Svelte frontend. Each command runs as
//! an async task (off the main thread); the bodies are blocking transport I/O
//! guarded by the connection mutex. Discovery and status changes also emit
//! events the UI listens for.

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::error::AppError;
use crate::protocol::{Request, Response};
use crate::state::AppState;
use crate::transport::{DeviceIdentity, DeviceInfo};

/// Payload for the `connection-status` event and connect/disconnect replies.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DeviceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<DeviceIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ConnectionStatus {
    fn disconnected() -> Self {
        Self { connected: false, device: None, identity: None, error: None }
    }
}

/// Scan every transport for candidate devices. Emits `device-found` per device
/// (so the UI can render progressively) and also returns the full list.
#[tauri::command]
pub async fn scan_devices(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<DeviceInfo>, AppError> {
    let devices = state.registry.discover_all();
    for d in &devices {
        let _ = app.emit("device-found", d.clone());
    }
    Ok(devices)
}

/// Open a device, run the identify handshake, and make it the active connection.
#[tauri::command]
pub async fn connect_device(
    app: AppHandle,
    state: State<'_, AppState>,
    device: DeviceInfo,
) -> Result<ConnectionStatus, AppError> {
    let mut guard = state.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    guard.clear();

    let mut transport = state
        .registry
        .make_transport(device.protocol)
        .ok_or_else(|| AppError::Unsupported(device.protocol.label().to_string()))?;

    let identity = transport
        .connect(&device)
        .map_err(|e| AppError::Connect(e.to_string()))?;

    guard.transport = Some(transport);
    guard.device = Some(device.clone());
    guard.identity = Some(identity.clone());
    drop(guard);

    let status = ConnectionStatus {
        connected: true,
        device: Some(device),
        identity: Some(identity),
        error: None,
    };
    let _ = app.emit("connection-status", status.clone());
    Ok(status)
}

/// Tear down the active connection.
#[tauri::command]
pub async fn disconnect_device(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ConnectionStatus, AppError> {
    {
        let mut guard = state.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        guard.clear();
    }
    let status = ConnectionStatus::disconnected();
    let _ = app.emit("connection-status", status.clone());
    Ok(status)
}

/// Send one protocol request over the active connection and return the reply.
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    request: Request,
) -> Result<Response, AppError> {
    let mut guard = state.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let transport = guard.transport.as_mut().ok_or(AppError::NotConnected)?;
    transport
        .request(&request)
        .map_err(|e| AppError::Request(e.to_string()))
}

/// Cheap snapshot of the current connection (for the frontend's heartbeat poll).
#[tauri::command]
pub async fn connection_status(
    state: State<'_, AppState>,
) -> Result<ConnectionStatus, AppError> {
    let guard = state.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ConnectionStatus {
        connected: guard.transport.is_some(),
        device: guard.device.clone(),
        identity: guard.identity.clone(),
        error: None,
    })
}
