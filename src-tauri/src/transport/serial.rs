//! Serial transport (crate `serialport`). Enumerates COM/tty ports, opens by
//! port name + baud, and speaks the newline-JSON wire protocol. Covers a
//! USB-CDC virtual COM port and a real UART via an FTDI/CP2102 adapter alike —
//! the same port handling proven in `serial-flash-gui`.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serialport::{SerialPort, SerialPortType};

use super::{Address, DeviceIdentity, DeviceInfo, Protocol, Transport};
use crate::protocol::{codec, Request, Response};

const DEFAULT_BAUD: u32 = 115_200;
const TIMEOUT: Duration = Duration::from_millis(1500);
/// Bound on noise/log lines skipped while waiting for a matching response.
const MAX_SKIP_LINES: usize = 256;

pub struct SerialTransport {
    conn: Option<Conn>,
}

struct Conn {
    /// Used for writing; reads go through `reader` (a clone of the same port).
    writer: Box<dyn SerialPort>,
    reader: Box<dyn SerialPort>,
    next_id: u64,
}

impl SerialTransport {
    pub fn new() -> Self {
        Self { conn: None }
    }
}

impl Default for SerialTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for SerialTransport {
    fn protocol(&self) -> Protocol {
        Protocol::Serial
    }

    fn discover(&self) -> Vec<DeviceInfo> {
        let ports = match serialport::available_ports() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };
        ports
            .into_iter()
            .map(|p| {
                let name = match &p.port_type {
                    SerialPortType::UsbPort(info) => match info.product.as_deref() {
                        Some(product) if !product.is_empty() => {
                            format!("{} — {}", p.port_name, product)
                        }
                        _ => format!("{} (USB serial)", p.port_name),
                    },
                    _ => p.port_name.clone(),
                };
                DeviceInfo {
                    id: format!("serial:{}", p.port_name),
                    protocol: Protocol::Serial,
                    name,
                    image: Protocol::Serial.image_key().into(),
                    address: Address::Port { name: p.port_name.clone(), baud: DEFAULT_BAUD },
                    identity: None,
                }
            })
            .collect()
    }

    fn connect(&mut self, device: &DeviceInfo) -> Result<DeviceIdentity> {
        let (port_name, baud) = match &device.address {
            Address::Port { name, baud } => (name.clone(), *baud),
            _ => return Err(anyhow!("serial transport requires a port address")),
        };

        let writer = serialport::new(&port_name, baud)
            .timeout(TIMEOUT)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .open()
            .map_err(|e| anyhow!("failed to open {}: {}", port_name, e))?;
        let reader = writer
            .try_clone()
            .map_err(|e| anyhow!("failed to clone {} for reading: {}", port_name, e))?;

        let mut conn = Conn { writer, reader, next_id: 1 };

        // Identify handshake confirms the device speaks our protocol.
        let resp = conn.roundtrip(&Request::Identify).map_err(|e| {
            anyhow!(
                "opened {} but no identify response — is the MidiController firmware \
                 running and speaking the wire protocol? ({})",
                port_name,
                e
            )
        })?;
        let identity: DeviceIdentity = resp
            .data
            .ok_or_else(|| anyhow!("identify returned no data"))
            .and_then(|d| {
                serde_json::from_value(d).map_err(|e| anyhow!("bad identify payload: {}", e))
            })?;

        self.conn = Some(conn);
        Ok(identity)
    }

    fn disconnect(&mut self) -> Result<()> {
        self.conn = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    fn request(&mut self, req: &Request) -> Result<Response> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| anyhow!("serial transport not connected"))?;
        conn.roundtrip(req)
    }
}

impl Conn {
    fn roundtrip(&mut self, req: &Request) -> Result<Response> {
        let id = self.next_id;
        self.next_id += 1;
        codec::roundtrip(&mut self.writer, &mut self.reader, id, req, MAX_SKIP_LINES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transport_is_disconnected() {
        let t = SerialTransport::new();
        assert!(!t.is_connected());
        assert_eq!(t.protocol(), Protocol::Serial);
    }

    #[test]
    fn discovered_ports_are_well_formed() {
        // Whatever ports the host has, each must look like a Serial DeviceInfo.
        for d in SerialTransport::new().discover() {
            assert_eq!(d.protocol, Protocol::Serial);
            assert!(d.id.starts_with("serial:"), "id was {}", d.id);
            assert_eq!(d.image, "serial");
            assert!(matches!(d.address, Address::Port { .. }));
            assert!(d.identity.is_none()); // not probed during discovery
        }
    }

    #[test]
    fn connect_rejects_non_port_address() {
        let mut t = SerialTransport::new();
        let device = DeviceInfo {
            id: "mock:0".into(),
            protocol: Protocol::Serial,
            name: "wrong".into(),
            image: "serial".into(),
            address: Address::Mock,
            identity: None,
        };
        let err = t.connect(&device).unwrap_err().to_string();
        assert!(err.contains("port address"), "got: {err}");
    }

    #[test]
    fn request_without_connection_errors() {
        let mut t = SerialTransport::new();
        assert!(t.request(&Request::Ping).is_err());
    }

    #[test]
    fn disconnect_is_idempotent() {
        let mut t = SerialTransport::new();
        assert!(t.disconnect().is_ok());
        assert!(t.disconnect().is_ok());
    }
}
