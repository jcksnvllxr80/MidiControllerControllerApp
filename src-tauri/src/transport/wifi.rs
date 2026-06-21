//! WiFi transport: the **same** newline-JSON protocol as serial, over a TCP
//! socket, with the controller discovered via mDNS. The firmware advertises
//! `_midicontroller._tcp` on port 8080 (host `midicontroller.local`) and answers
//! the identical framing on TCP that it does on USB — so `codec` is reused
//! verbatim and this is essentially `serial.rs` with a `TcpStream` + mDNS.

use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent};

use super::{Address, DeviceIdentity, DeviceInfo, Protocol, Transport};
use crate::protocol::{codec, Request, Response};

const SERVICE: &str = "_midicontroller._tcp.local.";
const DEFAULT_PORT: u16 = 8080;
const TIMEOUT: Duration = Duration::from_millis(1500);
/// How long a scan listens for mDNS replies (responders usually answer in <500ms).
const DISCOVER_WINDOW: Duration = Duration::from_millis(700);
/// Bound on noise/log lines skipped while waiting for a matching response.
const MAX_SKIP_LINES: usize = 256;
const HANDSHAKE_RETRIES: usize = 2;
const RETRY_DELAY: Duration = Duration::from_millis(200);

pub struct WifiTransport {
    conn: Option<Conn>,
}

struct Conn {
    writer: TcpStream,
    reader: TcpStream, // a clone of `writer`; reads go here (codec needs two handles)
    next_id: u64,
}

impl WifiTransport {
    pub fn new() -> Self {
        Self { conn: None }
    }
}

impl Default for WifiTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for WifiTransport {
    fn protocol(&self) -> Protocol {
        Protocol::Wifi
    }

    fn discover(&self) -> Vec<DeviceInfo> {
        let daemon = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        let recv = match daemon.browse(SERVICE) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut out: Vec<DeviceInfo> = Vec::new();
        let start = Instant::now();
        while start.elapsed() < DISCOVER_WINDOW {
            let remaining = DISCOVER_WINDOW.saturating_sub(start.elapsed());
            match recv.recv_timeout(remaining) {
                Ok(ServiceEvent::ServiceResolved(svc)) => {
                    let port = if svc.port != 0 { svc.port } else { DEFAULT_PORT };
                    let hostname = svc.host.trim_end_matches('.').to_string();
                    // Prefer a concrete IPv4 (Windows can't always resolve .local);
                    // fall back to the advertised hostname.
                    let host = svc
                        .addresses
                        .iter()
                        .map(|a| a.to_ip_addr())
                        .find(IpAddr::is_ipv4)
                        .map(|ip| ip.to_string())
                        .unwrap_or_else(|| hostname.clone());
                    let id = format!("wifi:{host}:{port}");
                    if out.iter().any(|d| d.id == id) {
                        continue;
                    }
                    out.push(DeviceInfo {
                        id,
                        protocol: Protocol::Wifi,
                        name: format!("{hostname} (Wi-Fi)"),
                        image: Protocol::Wifi.image_key().into(),
                        address: Address::Net { host, port },
                        identity: None,
                    });
                }
                Ok(_) => {}        // ServiceFound / SearchStarted / etc. — ignore
                Err(_) => break,   // recv timed out: discovery window is over
            }
        }
        let _ = daemon.shutdown();
        out
    }

    fn connect(&mut self, device: &DeviceInfo) -> Result<DeviceIdentity> {
        let (host, port) = match &device.address {
            Address::Net { host, port } => (host.clone(), *port),
            _ => return Err(anyhow!("wifi transport requires a net address")),
        };

        let writer = connect_timeout(&host, port, TIMEOUT)?;
        let _ = writer.set_read_timeout(Some(TIMEOUT));
        let _ = writer.set_write_timeout(Some(TIMEOUT));
        let reader = writer
            .try_clone()
            .map_err(|e| anyhow!("failed to clone {host}:{port} socket: {e}"))?;

        let mut conn = Conn { writer, reader, next_id: 1 };

        // Identify handshake confirms the device speaks our protocol.
        // Retry a couple of times — the firmware may still be flushing startup
        // output when the socket is first opened and miss the first request.
        let resp = {
            let mut last_err = anyhow!("no attempts made");
            let mut found = None;
            for attempt in 0..=HANDSHAKE_RETRIES {
                if attempt > 0 {
                    std::thread::sleep(RETRY_DELAY);
                }
                match conn.roundtrip(&Request::Identify) {
                    Ok(r) => { found = Some(r); break; }
                    Err(e) => last_err = e,
                }
            }
            found.ok_or_else(|| {
                anyhow!(
                    "connected to {host}:{port} but no identify response after {} attempts — \
                     is the MidiController firmware on this network and speaking the protocol? ({last_err})",
                    HANDSHAKE_RETRIES + 1
                )
            })?
        };
        let identity: DeviceIdentity = resp
            .data
            .ok_or_else(|| anyhow!("identify returned no data"))
            .and_then(|d| {
                serde_json::from_value(d).map_err(|e| anyhow!("bad identify payload: {e}"))
            })?;

        self.conn = Some(conn);
        Ok(identity)
    }

    fn disconnect(&mut self) -> Result<()> {
        self.conn = None; // dropping the streams closes the socket
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    fn request(&mut self, req: &Request) -> Result<Response> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| anyhow!("wifi transport not connected"))?;
        conn.roundtrip(req)
    }
}

/// Connect with a bounded timeout, trying each resolved address.
fn connect_timeout(host: &str, port: u16, timeout: Duration) -> Result<TcpStream> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| anyhow!("cannot resolve {host}:{port}: {e}"))?;
    let mut last: Option<std::io::Error> = None;
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => return Ok(s),
            Err(e) => last = Some(e),
        }
    }
    Err(anyhow!(
        "failed to connect to {host}:{port}: {}",
        last.map(|e| e.to_string()).unwrap_or_else(|| "no addresses resolved".into())
    ))
}

impl Conn {
    fn roundtrip(&mut self, req: &Request) -> Result<Response> {
        let id = self.next_id;
        self.next_id += 1;
        codec::roundtrip(&mut self.writer, &mut self.reader, id, req, MAX_SKIP_LINES, &mut |_| {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transport_is_disconnected() {
        let t = WifiTransport::new();
        assert!(!t.is_connected());
        assert_eq!(t.protocol(), Protocol::Wifi);
    }

    #[test]
    fn connect_rejects_non_net_address() {
        let mut t = WifiTransport::new();
        let device = DeviceInfo {
            id: "mock:0".into(),
            protocol: Protocol::Wifi,
            name: "x".into(),
            image: "wifi".into(),
            address: Address::Mock,
            identity: None,
        };
        let err = t.connect(&device).unwrap_err().to_string();
        assert!(err.contains("net address"), "got: {err}");
    }

    #[test]
    fn request_without_connection_errors() {
        let mut t = WifiTransport::new();
        assert!(t.request(&Request::Ping).is_err());
    }

    #[test]
    fn disconnect_is_idempotent() {
        let mut t = WifiTransport::new();
        assert!(t.disconnect().is_ok());
        assert!(t.disconnect().is_ok());
        assert!(!t.is_connected());
    }

    #[test]
    fn discovered_devices_are_well_formed() {
        // No device is required on the LAN; whatever resolves must be well-formed.
        for d in WifiTransport::new().discover() {
            assert_eq!(d.protocol, Protocol::Wifi);
            assert!(d.id.starts_with("wifi:"), "id was {}", d.id);
            assert_eq!(d.image, "wifi");
            assert!(matches!(d.address, Address::Net { .. }));
            assert!(d.identity.is_none());
        }
    }
}
