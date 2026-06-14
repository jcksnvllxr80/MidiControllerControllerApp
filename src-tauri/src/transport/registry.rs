//! Holds every concrete transport and fans discovery out across all of them.
//! This is what makes "scan to see which connection methods are available" a
//! single call, and what a new link family (Wi-Fi, Ethernet) plugs into.

use super::{
    mock::MockTransport, serial::SerialTransport, usb::UsbTransport, wifi::WifiTransport,
    DeviceInfo, Protocol, Transport,
};

pub struct TransportRegistry {
    /// Whether the in-memory mock device is offered (dev convenience).
    include_mock: bool,
}

impl TransportRegistry {
    pub fn new(include_mock: bool) -> Self {
        Self { include_mock }
    }

    pub fn with_defaults() -> Self {
        // Mock is on by default for development; set MIDICTRL_NO_MOCK to hide it.
        Self::new(std::env::var("MIDICTRL_NO_MOCK").is_err())
    }

    /// Enumerate candidate devices across all link families.
    pub fn discover_all(&self) -> Vec<DeviceInfo> {
        let mut out = Vec::new();
        out.extend(SerialTransport::new().discover());
        out.extend(UsbTransport::new().discover());
        out.extend(WifiTransport::new().discover());
        // Ethernet plugs in here once implemented.
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
            Protocol::Wifi => Some(Box::new(WifiTransport::new())),
            Protocol::Mock => Some(Box::new(MockTransport::new())),
            // Same trait, not yet implemented.
            Protocol::Ethernet => None,
        }
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_all_includes_mock_when_enabled() {
        let devices = TransportRegistry::new(true).discover_all();
        assert!(devices.iter().any(|d| d.id == "mock:0"));
    }

    #[test]
    fn discover_all_excludes_mock_when_disabled() {
        let devices = TransportRegistry::new(false).discover_all();
        assert!(!devices.iter().any(|d| d.id == "mock:0"));
    }

    #[test]
    fn discovered_devices_are_all_well_formed() {
        // Across serial + usb + wifi + mock, ids carry a protocol prefix and never panic.
        for d in TransportRegistry::new(true).discover_all() {
            assert!(
                d.id.starts_with("serial:")
                    || d.id.starts_with("usb:")
                    || d.id.starts_with("wifi:")
                    || d.id.starts_with("mock:"),
                "unexpected id {}",
                d.id
            );
        }
    }

    #[test]
    fn make_transport_maps_known_protocols() {
        let reg = TransportRegistry::new(true);
        assert!(reg.make_transport(Protocol::Serial).is_some());
        assert!(reg.make_transport(Protocol::Usb).is_some());
        assert!(reg.make_transport(Protocol::Wifi).is_some());
        assert!(reg.make_transport(Protocol::Mock).is_some());
    }

    #[test]
    fn make_transport_none_for_unimplemented() {
        let reg = TransportRegistry::new(true);
        assert!(reg.make_transport(Protocol::Ethernet).is_none());
    }

    #[test]
    fn made_transport_reports_its_protocol() {
        let reg = TransportRegistry::new(true);
        assert_eq!(reg.make_transport(Protocol::Serial).unwrap().protocol(), Protocol::Serial);
        assert_eq!(reg.make_transport(Protocol::Usb).unwrap().protocol(), Protocol::Usb);
        assert_eq!(reg.make_transport(Protocol::Mock).unwrap().protocol(), Protocol::Mock);
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn exactly_one_mock_device_when_enabled() {
        let count = TransportRegistry::new(true)
            .discover_all()
            .iter()
            .filter(|d| d.protocol == Protocol::Mock)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn no_mock_devices_when_disabled() {
        let count = TransportRegistry::new(false)
            .discover_all()
            .iter()
            .filter(|d| d.protocol == Protocol::Mock)
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn default_constructs_and_discovers_without_panic() {
        let _ = TransportRegistry::default().discover_all();
    }

    #[test]
    fn made_mock_transport_answers() {
        let mut t = TransportRegistry::new(true).make_transport(Protocol::Mock).unwrap();
        let dev = DeviceInfo {
            id: "mock:0".into(),
            protocol: Protocol::Mock,
            name: "m".into(),
            image: "mock".into(),
            address: crate::transport::Address::Mock,
            identity: None,
        };
        assert!(t.connect(&dev).is_ok());
        assert!(t.request(&crate::protocol::Request::Ping).unwrap().ok);
    }
}
