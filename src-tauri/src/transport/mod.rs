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
