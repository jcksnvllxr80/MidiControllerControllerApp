//! Shared application state: the transport registry plus the single active
//! connection, guarded by a mutex so commands can drive it safely.

use std::sync::{Arc, Mutex};

use crate::transport::{DeviceIdentity, DeviceInfo, Transport, TransportRegistry};

pub struct AppState {
    pub registry: Arc<TransportRegistry>,
    pub active: Arc<Mutex<ActiveConnection>>,
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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
