//! The transport interface: one trait every link family (Serial, USB, Wi-Fi,
//! Ethernet) implements, plus the value types the UI uses to render discovered
//! devices. This is the contract that lets the rest of the app stay ignorant of
//! *how* it's talking to the controller.

use serde::{Deserialize, Serialize};

pub mod mock;
pub mod registry;
pub mod serial;
pub mod usb;

pub use registry::TransportRegistry;

use crate::protocol::{Request, Response};

/// The link family a device speaks over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Serial,
    Usb,
    Wifi,
    Ethernet,
    /// In-memory device for development and tests.
    Mock,
}

impl Protocol {
    /// Human label shown on a device card.
    pub fn label(self) -> &'static str {
        match self {
            Protocol::Serial => "Serial",
            Protocol::Usb => "USB",
            Protocol::Wifi => "Wi-Fi",
            Protocol::Ethernet => "Ethernet",
            Protocol::Mock => "Mock",
        }
    }

    /// Asset key the frontend maps to a device image.
    pub fn image_key(self) -> &'static str {
        match self {
            Protocol::Serial => "serial",
            Protocol::Usb => "usb",
            Protocol::Wifi => "wifi",
            Protocol::Ethernet => "ethernet",
            Protocol::Mock => "mock",
        }
    }
}

/// How to physically reach a candidate device.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Address {
    Port { name: String, baud: u32 },
    Usb { vid: u16, pid: u16, serial: Option<String> },
    Net { host: String, port: u16 },
    Mock,
}

/// A device surfaced by a scan, before it's been confirmed as ours.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Stable handle, e.g. `serial:COM4` / `usb:1209:0001` / `mock:0`.
    pub id: String,
    pub protocol: Protocol,
    /// Human label (port name, USB product string, mDNS name).
    pub name: String,
    /// Asset key for the card image (see `Protocol::image_key`).
    pub image: String,
    pub address: Address,
    /// Filled in once an identify handshake succeeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<DeviceIdentity>,
}

/// What a device reports about itself after an identify handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub name: String,
    pub firmware: String,
    pub protocol_version: u16,
}

/// The one interface every link family implements.
///
/// Synchronous and `Send`: the active connection lives behind an
/// `Arc<Mutex<..>>` and is driven from Tauri's async command tasks. Keeping the
/// trait blocking mirrors the proven `serial-flash-gui` design and sidesteps
/// async-trait object/`Send` complexity.
pub trait Transport: Send {
    /// The link family this transport speaks (part of the interface; handy for
    /// logging and future routing even where `DeviceInfo` already carries it).
    #[allow(dead_code)]
    fn protocol(&self) -> Protocol;

    /// Enumerate candidate devices on this link. Does not open them.
    fn discover(&self) -> Vec<DeviceInfo>;

    /// Open `device`, run the identify handshake, and return its identity.
    /// Leaves the transport connected on success.
    fn connect(&mut self, device: &DeviceInfo) -> anyhow::Result<DeviceIdentity>;

    /// Close the active connection.
    fn disconnect(&mut self) -> anyhow::Result<()>;

    /// Send one request and block for the matching response.
    fn request(&mut self, req: &Request) -> anyhow::Result<Response>;

    #[allow(dead_code)]
    fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_labels_and_image_keys_cover_all_variants() {
        let all = [
            Protocol::Serial,
            Protocol::Usb,
            Protocol::Wifi,
            Protocol::Ethernet,
            Protocol::Mock,
        ];
        let labels: Vec<_> = all.iter().map(|p| p.label()).collect();
        assert_eq!(labels, ["Serial", "USB", "Wi-Fi", "Ethernet", "Mock"]);
        let keys: Vec<_> = all.iter().map(|p| p.image_key()).collect();
        assert_eq!(keys, ["serial", "usb", "wifi", "ethernet", "mock"]);
    }

    #[test]
    fn image_keys_are_unique() {
        let all = [
            Protocol::Serial,
            Protocol::Usb,
            Protocol::Wifi,
            Protocol::Ethernet,
            Protocol::Mock,
        ];
        let mut keys: Vec<_> = all.iter().map(|p| p.image_key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all.len());
    }

    #[test]
    fn protocol_serializes_lowercase() {
        assert_eq!(serde_json::to_value(Protocol::Serial).unwrap(), "serial");
        assert_eq!(serde_json::to_value(Protocol::Usb).unwrap(), "usb");
        assert_eq!(serde_json::to_value(Protocol::Wifi).unwrap(), "wifi");
        let p: Protocol = serde_json::from_str("\"ethernet\"").unwrap();
        assert_eq!(p, Protocol::Ethernet);
    }

    #[test]
    fn address_is_tagged_by_kind() {
        let v = serde_json::to_value(Address::Port { name: "COM4".into(), baud: 115200 }).unwrap();
        assert_eq!(v["kind"], "port");
        assert_eq!(v["name"], "COM4");
        assert_eq!(v["baud"], 115200);

        let v = serde_json::to_value(Address::Usb { vid: 0x1209, pid: 0x0001, serial: None }).unwrap();
        assert_eq!(v["kind"], "usb");
        assert_eq!(v["vid"], 0x1209);

        let v = serde_json::to_value(Address::Net { host: "h".into(), port: 80 }).unwrap();
        assert_eq!(v["kind"], "net");

        let v = serde_json::to_value(Address::Mock).unwrap();
        assert_eq!(v["kind"], "mock");
    }

    #[test]
    fn device_info_round_trips_and_omits_absent_identity() {
        let d = DeviceInfo {
            id: "serial:COM4".into(),
            protocol: Protocol::Serial,
            name: "COM4".into(),
            image: "serial".into(),
            address: Address::Port { name: "COM4".into(), baud: 115200 },
            identity: None,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert!(v.get("identity").is_none(), "absent identity must be omitted");

        let back: DeviceInfo = serde_json::from_value(v).unwrap();
        assert_eq!(back.id, "serial:COM4");
        assert_eq!(back.protocol, Protocol::Serial);
    }

    #[test]
    fn device_identity_round_trips() {
        let id = DeviceIdentity {
            name: "MidiController".into(),
            firmware: "1.2.3".into(),
            protocol_version: 1,
        };
        let back: DeviceIdentity = serde_json::from_value(serde_json::to_value(&id).unwrap()).unwrap();
        assert_eq!(back.name, "MidiController");
        assert_eq!(back.protocol_version, 1);
    }
}
