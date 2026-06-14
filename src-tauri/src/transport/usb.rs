//! Raw USB transport (crate `nusb`, a pure-Rust libusb alternative). Talks to a
//! firmware that exposes a **vendor-specific** USB interface with bulk IN/OUT
//! endpoints (rather than a CDC virtual COM port — that's the Serial transport).
//!
//! Contract with the firmware (`MidiControllerCpp`, build `-DMC_ENABLE_USB_EDITOR`):
//! a vendor interface carrying the SAME newline-delimited JSON protocol the
//! serial link uses. On Windows the interface must be bound to WinUSB (the
//! firmware ships MS-OS-2.0 descriptors so that happens automatically); without
//! that, `open`/`claim_interface` fail and we surface a clear hint.
//!
//! I/O reuses the shared `codec` by adapting the bulk endpoints to `std::io`.
//! Transfers run on a worker thread so we can impose a hard timeout (nusb 0.1
//! transfers have none of their own).

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use futures_lite::future::block_on;
use nusb::transfer::RequestBuffer;
use nusb::Interface;

use super::{Address, DeviceIdentity, DeviceInfo, Protocol, Transport};
use crate::protocol::{codec, Request, Response};

/// Firmware's vendor identifiers and endpoint layout — must match
/// `src/adapters/mcu/usb/usb_descriptors.c` in the firmware repo.
const VENDOR_VID: u16 = 0xCAFE;
const VENDOR_PID: u16 = 0x4001;
const IFACE: u8 = 0; // the vendor interface number
const EP_OUT: u8 = 0x01; // host -> device (bulk OUT)
const EP_IN: u8 = 0x81; // device -> host (bulk IN)

/// Read transfer size. The host stack coalesces 64-byte bulk packets and the
/// transfer returns on a short packet, so one transfer usually drains a whole
/// reply; large `get_pedal` payloads take a handful.
const READ_CHUNK: usize = 4096;
const TIMEOUT: Duration = Duration::from_millis(1500);
/// Bound on noise/log lines skipped while waiting for a matching response.
const MAX_SKIP_LINES: usize = 256;

/// Narrow discovery to the firmware's vendor VID/PID so the picker shows our
/// device only — not every USB peripheral (which all dead-ended before).
const FIRMWARE_FILTER: Option<(u16, u16)> = Some((VENDOR_VID, VENDOR_PID));

pub struct UsbTransport {
    link: Option<UsbLink>,
}

impl UsbTransport {
    pub fn new() -> Self {
        Self { link: None }
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

    fn connect(&mut self, device: &DeviceInfo) -> Result<DeviceIdentity> {
        let (vid, pid, serial) = match &device.address {
            Address::Usb { vid, pid, serial } => (*vid, *pid, serial.clone()),
            _ => return Err(anyhow!("usb transport requires a usb address")),
        };

        let info = nusb::list_devices()
            .map_err(|e| anyhow!("USB enumeration failed: {e}"))?
            .find(|d| {
                d.vendor_id() == vid
                    && d.product_id() == pid
                    && serial.as_deref().map_or(true, |s| d.serial_number() == Some(s))
            })
            .ok_or_else(|| anyhow!("USB device {vid:04x}:{pid:04x} not found (unplugged?)"))?;

        let dev = info.open().map_err(|e| {
            anyhow!(
                "failed to open USB device {vid:04x}:{pid:04x}: {e}. On Windows the vendor \
                 interface must be bound to WinUSB — confirm the firmware was built with \
                 -DMC_ENABLE_USB_EDITOR (it ships MS-OS-2.0 descriptors for auto-binding)."
            )
        })?;

        // On Linux a kernel driver may hold the interface; detach if needed.
        let iface = dev
            .claim_interface(IFACE)
            .or_else(|_| dev.detach_and_claim_interface(IFACE))
            .map_err(|e| anyhow!("failed to claim USB interface {IFACE}: {e}"))?;

        let mut link = UsbLink::new(iface);

        // Identify handshake confirms the device speaks our protocol.
        let resp = link.roundtrip(&Request::Identify).map_err(|e| {
            anyhow!(
                "claimed the USB interface but no identify response — is the firmware's \
                 vendor link running and speaking the wire protocol? ({e})"
            )
        })?;
        let identity: DeviceIdentity = resp
            .data
            .ok_or_else(|| anyhow!("identify returned no data"))
            .and_then(|d| serde_json::from_value(d).map_err(|e| anyhow!("bad identify payload: {e}")))?;

        self.link = Some(link);
        Ok(identity)
    }

    fn disconnect(&mut self) -> Result<()> {
        self.link = None; // dropping the Interface releases it / cancels transfers
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.link.is_some()
    }

    fn request(&mut self, req: &Request) -> Result<Response> {
        let link = self
            .link
            .as_mut()
            .ok_or_else(|| anyhow!("usb transport not connected"))?;
        link.roundtrip(req)
    }
}

/// One claimed vendor interface, adapted to `std::io` so the shared `codec` can
/// frame requests/responses over its bulk endpoints.
struct UsbLink {
    iface: Interface,
    rx: VecDeque<u8>,
    next_id: u64,
}

impl UsbLink {
    fn new(iface: Interface) -> Self {
        Self { iface, rx: VecDeque::new(), next_id: 1 }
    }

    fn roundtrip(&mut self, req: &Request) -> Result<Response> {
        let id = self.next_id;
        self.next_id += 1;
        let bytes = codec::encode_request(id, req)?;
        self.write_all(&bytes)?;
        self.flush()?;
        for _ in 0..MAX_SKIP_LINES {
            let line = codec::read_line(self)?;
            if let Some(resp) = codec::match_response_line(&line, id)? {
                return Ok(resp);
            }
        }
        Err(anyhow!("no matching response for request {id}"))
    }
}

/// Marker error so the `io::Read` adapter can map a transfer timeout onto
/// `ErrorKind::TimedOut` (which `codec::read_line` turns into a clean message).
const TIMEOUT_MARK: &str = "__usb_timeout__";

/// Run one bulk transfer on a worker thread with a hard timeout. `data` present
/// => bulk OUT (returns empty); absent => bulk IN (returns the bytes read).
fn bulk(iface: &Interface, data: Option<Vec<u8>>, timeout: Duration) -> Result<Vec<u8>> {
    let iface = iface.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let res = match data {
            Some(buf) => block_on(iface.bulk_out(EP_OUT, buf))
                .into_result()
                .map(|_| Vec::new())
                .map_err(|e| anyhow!("USB bulk OUT failed: {e}")),
            None => block_on(iface.bulk_in(EP_IN, RequestBuffer::new(READ_CHUNK)))
                .into_result()
                .map_err(|e| anyhow!("USB bulk IN failed: {e}")),
        };
        let _ = tx.send(res);
    });
    match rx.recv_timeout(timeout) {
        Ok(r) => r,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(anyhow!(TIMEOUT_MARK)),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow!("USB worker thread died")),
    }
}

impl Read for UsbLink {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.rx.is_empty() {
            match bulk(&self.iface, None, TIMEOUT) {
                Ok(data) => self.rx.extend(data),
                Err(e) if e.to_string().contains(TIMEOUT_MARK) => {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "USB read timed out"));
                }
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
        }
        let n = out.len().min(self.rx.len());
        for slot in out.iter_mut().take(n) {
            *slot = self.rx.pop_front().unwrap();
        }
        Ok(n)
    }
}

impl Write for UsbLink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        bulk(&self.iface, Some(buf.to_vec()), TIMEOUT)
            .map(|_| buf.len())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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
        // Only our vendor VID/PID is listed now (FIRMWARE_FILTER), but whatever
        // is present must be a well-formed USB DeviceInfo.
        for d in UsbTransport::new().discover() {
            assert_eq!(d.protocol, Protocol::Usb);
            assert!(d.id.starts_with("usb:"), "id was {}", d.id);
            assert_eq!(d.image, "usb");
            assert!(matches!(d.address, Address::Usb { .. }));
            assert!(d.identity.is_none());
        }
    }

    #[test]
    fn connect_requires_usb_address() {
        let mut t = UsbTransport::new();
        let device = DeviceInfo {
            id: "mock:0".into(),
            protocol: Protocol::Usb,
            name: "x".into(),
            image: "usb".into(),
            address: Address::Mock,
            identity: None,
        };
        let err = t.connect(&device).unwrap_err().to_string();
        assert!(err.contains("usb address"), "got: {err}");
    }

    #[test]
    fn connect_absent_device_errors_cleanly() {
        // No hardware in tests: connecting to a made-up device must error (not
        // hang, not panic) and leave the transport disconnected.
        let mut t = UsbTransport::new();
        let device = DeviceInfo {
            id: "usb:1209:0001".into(),
            protocol: Protocol::Usb,
            name: "x".into(),
            image: "usb".into(),
            address: Address::Usb { vid: 0x1209, pid: 0x0001, serial: None },
            identity: None,
        };
        assert!(t.connect(&device).is_err());
        assert!(!t.is_connected());
    }

    #[test]
    fn request_without_connection_errors() {
        let mut t = UsbTransport::new();
        assert!(t.request(&Request::Ping).is_err());
    }

    #[test]
    fn disconnect_ok_and_idempotent() {
        let mut t = UsbTransport::new();
        assert!(t.disconnect().is_ok());
        assert!(t.disconnect().is_ok());
        assert!(!t.is_connected());
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn discovered_ids_have_vid_pid_shape() {
        for d in UsbTransport::new().discover() {
            let parts: Vec<&str> = d.id.split(':').collect();
            assert_eq!(parts.len(), 3, "id {}", d.id);
            assert_eq!(parts[0], "usb");
        }
    }

    #[test]
    fn default_is_disconnected() {
        assert!(!UsbTransport::default().is_connected());
    }

    #[test]
    fn firmware_filter_matches_descriptor_constants() {
        // Guard against the filter and endpoint contract drifting from firmware.
        assert_eq!(FIRMWARE_FILTER, Some((0xCAFE, 0x4001)));
        assert_eq!(EP_OUT, 0x01);
        assert_eq!(EP_IN, 0x81);
    }
}
