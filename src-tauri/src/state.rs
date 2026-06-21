//! Shared application state: the transport registry plus the single active
//! connection, guarded by a mutex. The connect/disconnect/send/status logic
//! lives here (hardware-free against the mock) so it's unit-testable; the Tauri
//! commands are thin wrappers that call these and emit events.

use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::error::AppError;
use crate::protocol::{Request, Response};
use crate::transport::{DeviceIdentity, DeviceInfo, Transport, TransportRegistry};

/// Snapshot of the connection, sent to the frontend on connect/disconnect and
/// via the `connection-status` event.
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
    pub fn disconnected() -> Self {
        Self { connected: false, device: None, identity: None, error: None }
    }
}

pub struct AppState {
    pub registry: Arc<TransportRegistry>,
    pub active: Arc<Mutex<ActiveConnection>>,
    /// Accumulates non-JSON lines (firmware printf output) captured during
    /// protocol exchanges. Drained by the command layer and emitted as events.
    pub log_buf: Arc<Mutex<Vec<String>>>,
    /// Directory where app.log and device.log are written.  Empty in test
    /// contexts (logging is not initialised there).
    pub log_dir: PathBuf,
}

#[derive(Default)]
pub struct ActiveConnection {
    pub transport: Option<Box<dyn Transport>>,
    pub device: Option<DeviceInfo>,
    pub identity: Option<DeviceIdentity>,
}

impl ActiveConnection {
    pub fn clear(&mut self) {
        if let Some(t) = self.transport.as_mut() {
            let _ = t.disconnect();
        }
        self.transport = None;
        self.device = None;
        self.identity = None;
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(TransportRegistry::with_defaults()),
            active: Arc::new(Mutex::new(ActiveConnection::default())),
            log_buf: Arc::new(Mutex::new(Vec::new())),
            log_dir: PathBuf::new(),
        }
    }

    /// Set the log directory (builder style).  Called by `lib.rs::run()` after
    /// `logging::init` so the path can be surfaced to the frontend on demand.
    pub fn with_log_dir(mut self, dir: PathBuf) -> Self {
        self.log_dir = dir;
        self
    }

    /// Return the active log directory as a string (empty if not configured).
    pub fn get_log_dir(&self) -> String {
        self.log_dir.to_string_lossy().into_owned()
    }

    /// Drain and return any log lines captured since the last drain.
    pub fn drain_logs(&self) -> Vec<String> {
        match self.log_buf.lock() {
            Ok(mut buf) => mem::take(&mut *buf),
            Err(_) => Vec::new(),
        }
    }

    /// Open a device, run the identify handshake, and make it the active
    /// connection (replacing any existing one).
    pub fn connect(&self, device: DeviceInfo) -> Result<ConnectionStatus, AppError> {
        let mut guard = self.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        guard.clear();

        let mut transport = self
            .registry
            .make_transport(device.protocol)
            .ok_or_else(|| AppError::Unsupported(device.protocol.label().to_string()))?;

        transport.set_log_sink(self.log_buf.clone());

        let identity = transport
            .connect(&device)
            .map_err(|e| AppError::Connect(e.to_string()))?;

        guard.transport = Some(transport);
        guard.device = Some(device.clone());
        guard.identity = Some(identity.clone());

        Ok(ConnectionStatus {
            connected: true,
            device: Some(device),
            identity: Some(identity),
            error: None,
        })
    }

    /// Tear down the active connection.
    pub fn disconnect(&self) -> Result<ConnectionStatus, AppError> {
        let mut guard = self.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        guard.clear();
        Ok(ConnectionStatus::disconnected())
    }

    /// Send one request over the active connection.
    pub fn send(&self, request: Request) -> Result<Response, AppError> {
        let mut guard = self.active.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let transport = guard.transport.as_mut().ok_or(AppError::NotConnected)?;
        transport
            .request(&request)
            .map_err(|e| AppError::Request(e.to_string()))
    }

    /// Cheap snapshot of the current connection.
    pub fn status(&self) -> ConnectionStatus {
        match self.active.lock() {
            Ok(guard) => ConnectionStatus {
                connected: guard.transport.is_some(),
                device: guard.device.clone(),
                identity: guard.identity.clone(),
                error: None,
            },
            Err(_) => ConnectionStatus::disconnected(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{mock::MockTransport, Address, Protocol};

    /// A real mock DeviceInfo straight from the mock transport's discovery.
    fn mock_device() -> DeviceInfo {
        MockTransport::new().discover().pop().expect("mock discovers one device")
    }

    fn ethernet_device() -> DeviceInfo {
        DeviceInfo {
            id: "ethernet:0".into(),
            protocol: Protocol::Ethernet,
            name: "future".into(),
            image: "ethernet".into(),
            address: Address::Net { host: "10.0.0.5".into(), port: 80 },
            identity: None,
        }
    }

    #[test]
    fn fresh_state_is_disconnected() {
        let s = AppState::new();
        let st = s.status();
        assert!(!st.connected);
        assert!(st.device.is_none());
    }

    #[test]
    fn connect_mock_reports_identity() {
        let s = AppState::new();
        let st = s.connect(mock_device()).unwrap();
        assert!(st.connected);
        assert_eq!(st.identity.unwrap().name, "Mock MidiController");
        assert!(s.status().connected);
    }

    #[test]
    fn send_after_connect_returns_data() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        let resp = s.send(Request::ListSets).unwrap();
        assert!(resp.ok);
        let names = resp.data.unwrap();
        assert!(names.as_array().unwrap().iter().any(|v| v == "Friday Gig"));
    }

    #[test]
    fn logical_error_is_ok_response_not_apperror() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        // Unknown set => transport returns ok=false, not a Rust error.
        let resp = s.send(Request::GetSet { name: "missing".into() }).unwrap();
        assert!(!resp.ok);
        assert!(resp.error.is_some());
    }

    #[test]
    fn send_without_connection_is_not_connected() {
        let s = AppState::new();
        match s.send(Request::Ping) {
            Err(AppError::NotConnected) => {}
            other => panic!("expected NotConnected, got {other:?}"),
        }
    }

    #[test]
    fn disconnect_clears_connection() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        let st = s.disconnect().unwrap();
        assert!(!st.connected);
        assert!(!s.status().connected);
        assert!(s.send(Request::Ping).is_err());
    }

    #[test]
    fn connect_unsupported_protocol_errors() {
        let s = AppState::new();
        match s.connect(ethernet_device()) {
            Err(AppError::Unsupported(p)) => assert_eq!(p, "Ethernet"),
            other => panic!("expected Unsupported, got {other:?}"),
        }
        assert!(!s.status().connected);
    }

    #[test]
    fn reconnect_replaces_previous_connection() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        s.connect(mock_device()).unwrap();
        assert!(s.status().connected);
    }

    #[test]
    fn disconnected_status_serializes_without_optional_fields() {
        let json = serde_json::to_value(ConnectionStatus::disconnected()).unwrap();
        assert_eq!(json["connected"], false);
        assert!(json.get("device").is_none());
        assert!(json.get("identity").is_none());
        assert!(json.get("error").is_none());
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use crate::transport::{mock::MockTransport, Address, Protocol};

    fn mock_device() -> DeviceInfo {
        MockTransport::new().discover().pop().unwrap()
    }

    #[test]
    fn status_reflects_connected_device_and_identity() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        let st = s.status();
        assert!(st.connected);
        assert_eq!(st.device.unwrap().id, "mock:0");
        assert_eq!(st.identity.unwrap().firmware, "sim-0.1");
    }

    #[test]
    fn write_then_get_via_state() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        s.send(Request::WriteSet { name: "Q".into(), data: serde_json::json!({ "songs": [] }) })
            .unwrap();
        assert!(s.send(Request::GetSet { name: "Q".into() }).unwrap().ok);
    }

    #[test]
    fn many_sends_succeed() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        for _ in 0..25 {
            assert!(s.send(Request::ListSongs).unwrap().ok);
        }
    }

    #[test]
    fn disconnect_when_idle_is_ok() {
        let s = AppState::new();
        assert!(!s.disconnect().unwrap().connected);
    }

    #[test]
    fn connect_ethernet_is_unsupported() {
        let s = AppState::new();
        let dev = DeviceInfo {
            id: "eth:0".into(),
            protocol: Protocol::Ethernet,
            name: "e".into(),
            image: "ethernet".into(),
            address: Address::Net { host: "h".into(), port: 1 },
            identity: None,
        };
        match s.connect(dev) {
            Err(AppError::Unsupported(p)) => assert_eq!(p, "Ethernet"),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn status_is_independent_snapshot() {
        let s = AppState::new();
        s.connect(mock_device()).unwrap();
        let earlier = s.status();
        s.disconnect().unwrap();
        assert!(earlier.connected);
        assert!(!s.status().connected);
    }
}
