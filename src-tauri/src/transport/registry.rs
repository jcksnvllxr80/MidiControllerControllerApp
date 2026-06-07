//! Holds every concrete transport and fans discovery out across all of them.
//! This is what makes "scan to see which connection methods are available" a
//! single call, and what a new link family (Wi-Fi, Ethernet) plugs into.

use super::{
    mock::MockTransport, serial::SerialTransport, usb::UsbTransport, DeviceInfo, Protocol,
    Transport,
};

pub struct TransportRegistry {
    /// Whether the in-memory mock device is offered (dev convenience).
    include_mock: bool,
}

impl TransportRegistry {
    pub fn with_defaults() -> Self {
        // Mock is on by default for development; set MIDICTRL_NO_MOCK to hide it.
        let include_mock = std::env::var("MIDICTRL_NO_MOCK").is_err();
        Self { include_mock }
    }

    /// Enumerate candidate devices across all link families.
    pub fn discover_all(&self) -> Vec<DeviceInfo> {
        let mut out = Vec::new();
        out.extend(SerialTransport::new().discover());
        out.extend(UsbTransport::new().discover());
        // Wi-Fi / Ethernet transports plug in here once implemented.
        if self.include_mock {
            out.extend(MockTransport::new().discover());
        }
        out
    }

    /// Build a fresh, unconnected transport for a device's protocol.
    pub fn make_transport(&self, protocol: Protocol) -> Option<Box<dyn Transport>> {
        match protocol {
            Protocol::Serial => Some(Box::new(SerialTransport::new())),
            Protocol::Usb => Some(Box::new(UsbTransport::new())),
            Protocol::Mock => Some(Box::new(MockTransport::new())),
            // Same trait, not yet implemented.
            Protocol::Wifi | Protocol::Ethernet => None,
        }
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
