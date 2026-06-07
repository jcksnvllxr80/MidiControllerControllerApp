//! Raw USB transport (crate `nusb`, pure-Rust libusb alternative). Enumerates
//! devices by VID/PID — for a firmware that exposes a vendor-specific USB
//! interface rather than a CDC virtual COM port.
//!
//! Discovery works today. `connect`/`request` are deliberately stubbed until the
//! firmware's USB descriptor is finalized: if the firmware enumerates as CDC,
//! these collapse onto the Serial transport and this file stays a stub; if it
//! exposes a vendor/HID interface, the endpoint I/O lands here. See
//! `docs/plan.md` Phase 3 and Open Question #1.

use anyhow::{anyhow, Result};

use super::{Address, DeviceIdentity, DeviceInfo, Protocol, Transport};
use crate::protocol::{Request, Response};

/// Narrow USB discovery to the firmware's VID/PID once it's known. While `None`,
/// discovery lists every named USB device so the scan demonstrates USB presence.
const FIRMWARE_FILTER: Option<(u16, u16)> = None;

pub struct UsbTransport {
    connected: bool,
}

impl UsbTransport {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for UsbTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for UsbTransport {
    fn protocol(&self) -> Protocol {
        Protocol::Usb
    }

    fn discover(&self) -> Vec<DeviceInfo> {
        let devices = match nusb::list_devices() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        let mut out = Vec::new();
        for d in devices {
            let vid = d.vendor_id();
            let pid = d.product_id();

            match FIRMWARE_FILTER {
                Some((fv, fp)) if vid != fv || pid != fp => continue,
                // Without a filter, skip anonymous devices (hubs/controllers).
                None if d.product_string().is_none() => continue,
                _ => {}
            }

            let name = d
                .product_string()
                .map(str::to_string)
                .unwrap_or_else(|| format!("USB {vid:04x}:{pid:04x}"));

            out.push(DeviceInfo {
                id: format!("usb:{vid:04x}:{pid:04x}"),
                protocol: Protocol::Usb,
                name,
                image: Protocol::Usb.image_key().into(),
                address: Address::Usb {
                    vid,
                    pid,
                    serial: d.serial_number().map(str::to_string),
                },
                identity: None,
            });
        }
        out
    }

    fn connect(&mut self, _device: &DeviceInfo) -> Result<DeviceIdentity> {
        Err(anyhow!(
            "USB raw transport: connect is pending the firmware's USB descriptor \
             (CDC virtual-COM vs vendor/HID). Discovery works today. If the firmware \
             enumerates as a CDC virtual COM port, connect via the Serial transport instead. \
             See docs/plan.md Phase 3."
        ))
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn request(&mut self, _req: &Request) -> Result<Response> {
        Err(anyhow!(
            "USB raw transport request not implemented yet (pending firmware USB descriptor)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transport_basics() {
        let t = UsbTransport::new();
        assert_eq!(t.protocol(), Protocol::Usb);
        assert!(!t.is_connected());
    }

    #[test]
    fn discovered_devices_are_well_formed() {
        for d in UsbTransport::new().discover() {
            assert_eq!(d.protocol, Protocol::Usb);
            assert!(d.id.starts_with("usb:"), "id was {}", d.id);
            assert_eq!(d.image, "usb");
            assert!(matches!(d.address, Address::Usb { .. }));
            assert!(d.identity.is_none());
        }
    }

    #[test]
    fn connect_is_a_clear_stub() {
        let mut t = UsbTransport::new();
        let device = DeviceInfo {
            id: "usb:1209:0001".into(),
            protocol: Protocol::Usb,
            name: "x".into(),
            image: "usb".into(),
            address: Address::Usb { vid: 0x1209, pid: 0x0001, serial: None },
            identity: None,
        };
        let err = t.connect(&device).unwrap_err().to_string();
        assert!(err.contains("pending"), "got: {err}");
        assert!(!t.is_connected());
    }

    #[test]
    fn request_is_a_stub() {
        let mut t = UsbTransport::new();
        assert!(t.request(&Request::Ping).is_err());
    }

    #[test]
    fn disconnect_ok() {
        let mut t = UsbTransport::new();
        assert!(t.disconnect().is_ok());
    }
}
