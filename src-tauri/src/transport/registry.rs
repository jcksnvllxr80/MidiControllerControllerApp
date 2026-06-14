//! Holds every concrete transport and fans discovery out across all of them.
//! This is what makes "scan to see which connection methods are available" a
//! single call, and what a new link family (Wi-Fi, Ethernet) plugs into.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use super::{
    mock::MockTransport, serial::SerialTransport, usb::UsbTransport, wifi::WifiTransport,
    DeviceIdentity, DeviceInfo, Protocol, Transport,
};

pub struct TransportRegistry {
    /// Whether the in-memory mock device is offered (dev convenience).
    include_mock: bool,
    /// Probe result per candidate id, so a present device is opened/identified
    /// once — not on every background scan (which would thrash the serial port).
    /// Pruned when a candidate disappears, so unplug/replug re-probes.
    identity_cache: Mutex<HashMap<String, Option<DeviceIdentity>>>,
}

impl TransportRegistry {
    pub fn new(include_mock: bool) -> Self {
        Self { include_mock, identity_cache: Mutex::new(HashMap::new()) }
    }

    pub fn with_defaults() -> Self {
        // Mock is on by default for development; set MIDICTRL_NO_MOCK to hide it.
        Self::new(std::env::var("MIDICTRL_NO_MOCK").is_err())
    }

    /// Discover real controllers across all link families. Candidates from each
    /// transport are pre-filtered to our VID/PID(/mDNS service), confirmed by an
    /// `identify` handshake (so only true MidiControllers survive), and merged by
    /// `device_id` so one physical unit seen on USB *and* WiFi is a single entry.
    pub fn discover_all(&self) -> Vec<DeviceInfo> {
        let mut candidates = Vec::new();
        candidates.extend(SerialTransport::new().discover());
        candidates.extend(UsbTransport::new().discover());
        candidates.extend(WifiTransport::new().discover());
        // Ethernet plugs in here once implemented.

        // Probe each candidate once (cached), then prune cache for ones gone.
        let mut cache = self.identity_cache.lock().unwrap();
        let present: HashSet<String> = candidates.iter().map(|c| c.id.clone()).collect();
        cache.retain(|id, _| present.contains(id));

        let mut confirmed: Vec<DeviceInfo> = Vec::new();
        for mut c in candidates {
            let result = match cache.get(&c.id) {
                Some(cached) => cached.clone(),
                None => {
                    let r = self.probe(&c);
                    cache.insert(c.id.clone(), r.clone());
                    r
                }
            };
            match result {
                Some(identity) if identity.name == "MidiController" => {
                    c.identity = Some(identity);
                    confirmed.push(c);
                }
                Some(_) => {} // something answered, but it isn't a MidiController — drop it
                // No reply: the candidate is already VID/mDNS-filtered to our
                // family, so keep it visible (unconfirmed) rather than hiding the
                // user's plugged-in device; the real connect identifies it.
                None => confirmed.push(c),
            }
        }
        drop(cache);

        // The in-memory mock is a trusted dev fixture — include without probing.
        if self.include_mock {
            confirmed.extend(MockTransport::new().discover());
        }

        dedupe_by_device_id(confirmed)
    }

    /// Open a candidate, run `identify`, and close. Used only to confirm/identify
    /// during discovery; the real session reconnects on demand. Candidates are
    /// already VID/PID-filtered, so this never touches an unrelated port.
    fn probe(&self, device: &DeviceInfo) -> Option<DeviceIdentity> {
        let mut t = self.make_transport(device.protocol)?;
        let identity = t.connect(device).ok();
        let _ = t.disconnect();
        identity
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

/// Merge devices that are the same physical unit (equal `device_id`) seen on
/// multiple transports into one entry, preferring a wired link (serial > usb >
/// wifi) — it's more reliable and is what firmware flashing needs. A device with
/// no `device_id` (older firmware) can't be matched, so it's kept as-is.
pub fn dedupe_by_device_id(devices: Vec<DeviceInfo>) -> Vec<DeviceInfo> {
    fn rank(p: Protocol) -> u8 {
        match p {
            Protocol::Serial => 0,
            Protocol::Usb => 1,
            Protocol::Wifi => 2,
            Protocol::Ethernet => 3,
            Protocol::Mock => 4,
        }
    }
    fn device_id_of(d: &DeviceInfo) -> Option<&str> {
        d.identity.as_ref().and_then(|i| i.device_id.as_deref())
    }

    let mut out: Vec<DeviceInfo> = Vec::new();
    for d in devices {
        match device_id_of(&d) {
            Some(id) => {
                let id = id.to_string();
                if let Some(existing) = out.iter_mut().find(|e| device_id_of(e) == Some(id.as_str())) {
                    if rank(d.protocol) < rank(existing.protocol) {
                        *existing = d; // a more-preferred transport for the same unit
                    }
                } else {
                    out.push(d);
                }
            }
            None => out.push(d),
        }
    }
    out
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

    fn dev(protocol: Protocol, device_id: Option<&str>, id: &str) -> DeviceInfo {
        DeviceInfo {
            id: id.into(),
            protocol,
            name: "MidiController".into(),
            image: "x".into(),
            address: crate::transport::Address::Mock,
            identity: Some(DeviceIdentity {
                name: "MidiController".into(),
                firmware: "1".into(),
                protocol_version: 1,
                device_id: device_id.map(str::to_string),
            }),
        }
    }

    #[test]
    fn dedupe_merges_same_unit_across_transports_preferring_wired() {
        let out = dedupe_by_device_id(vec![
            dev(Protocol::Wifi, Some("UNIT-A"), "wifi:a"),
            dev(Protocol::Serial, Some("UNIT-A"), "serial:a"),
            dev(Protocol::Usb, Some("UNIT-B"), "usb:b"),
        ]);
        assert_eq!(out.len(), 2, "UNIT-A's two transports collapse to one");
        let unit_a = out
            .iter()
            .find(|d| d.identity.as_ref().unwrap().device_id.as_deref() == Some("UNIT-A"))
            .unwrap();
        assert_eq!(unit_a.protocol, Protocol::Serial, "wired preferred over wifi");
    }

    #[test]
    fn dedupe_keeps_devices_without_a_device_id_separate() {
        let out = dedupe_by_device_id(vec![
            dev(Protocol::Serial, None, "serial:x"),
            dev(Protocol::Serial, None, "serial:y"),
        ]);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn dedupe_keeps_distinct_units() {
        let out = dedupe_by_device_id(vec![
            dev(Protocol::Serial, Some("A"), "serial:a"),
            dev(Protocol::Wifi, Some("B"), "wifi:b"),
        ]);
        assert_eq!(out.len(), 2);
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
