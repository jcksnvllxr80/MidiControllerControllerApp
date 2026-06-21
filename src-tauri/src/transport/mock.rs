//! In-memory transport: a fake controller that answers the full protocol from
//! seeded data. Lets the entire UI run end-to-end with no hardware, and backs
//! the protocol tests. Mirrors the web app's Set / Song / Pedal shapes.

use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::{json, Value};

use super::{Address, DeviceIdentity, DeviceInfo, Protocol, Transport};
use crate::protocol::{Request, Response};

pub struct MockTransport {
    connected: bool,
    sets: BTreeMap<String, Value>,
    songs: BTreeMap<String, Value>,
    pedals: BTreeMap<String, Value>,
    wifi_enabled: bool,
    wifi_connected: bool,
    wifi_ssid: String,
    wifi_ip: String,
    display: String,
}

impl MockTransport {
    pub fn new() -> Self {
        let mut t = Self {
            connected: false,
            sets: BTreeMap::new(),
            songs: BTreeMap::new(),
            pedals: BTreeMap::new(),
            wifi_enabled: false,
            wifi_connected: false,
            wifi_ssid: String::new(),
            wifi_ip: String::new(),
            display: String::new(),
        };
        t.seed();
        t
    }

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            name: "Mock MidiController".to_string(),
            firmware: "sim-0.1".to_string(),
            protocol_version: 1,
            device_id: Some("MOCK-0001".to_string()),
        }
    }

    fn seed(&mut self) {
        self.pedals.insert(
            "Timeline".into(),
            json!({ "name": "Timeline", "presets": [0, 1, 2, 3], "params": ["mix", "feedback"] }),
        );
        self.pedals.insert(
            "BigSky".into(),
            json!({ "name": "BigSky", "presets": [0, 1, 2], "params": ["decay", "tone"] }),
        );
        self.pedals.insert(
            "El Capistan".into(),
            json!({ "name": "El Capistan", "presets": [0, 1], "params": ["wow", "flutter"] }),
        );

        self.songs.insert(
            "Intro".into(),
            json!({
                "name": "Intro", "tempo": 120,
                "parts": { "A": { "position": 0, "pedals": {
                    "Timeline": { "engaged": true, "preset": 1, "params": {} } } } }
            }),
        );
        self.songs.insert(
            "Main Riff".into(),
            json!({
                "name": "Main Riff", "tempo": 140,
                "parts": { "A": { "position": 0, "pedals": {
                    "BigSky": { "engaged": true, "preset": 0, "params": {} } } } }
            }),
        );
        self.songs.insert(
            "Ballad".into(),
            json!({
                "name": "Ballad", "tempo": 72,
                "parts": { "A": { "position": 0, "pedals": {
                    "El Capistan": { "engaged": true, "preset": 0, "params": {} } } } }
            }),
        );

        self.sets.insert(
            "Friday Gig".into(),
            json!({ "name": "Friday Gig", "songs": ["Intro", "Main Riff"] }),
        );
        self.sets.insert(
            "Acoustic Set".into(),
            json!({ "name": "Acoustic Set", "songs": ["Ballad"] }),
        );
    }

    fn names(map: &BTreeMap<String, Value>) -> Value {
        Value::Array(map.keys().cloned().map(Value::from).collect())
    }

    fn get(map: &BTreeMap<String, Value>, name: &str, kind: &str) -> Response {
        match map.get(name) {
            Some(v) => Response::ok(v.clone()),
            None => Response::err(format!("no {kind} '{name}'")),
        }
    }

    fn wifi_status(&self) -> Value {
        json!({
            "enabled": self.wifi_enabled,
            "connected": self.wifi_connected,
            "ssid": self.wifi_ssid,
            "ip": self.wifi_ip,
        })
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for MockTransport {
    fn protocol(&self) -> Protocol {
        Protocol::Mock
    }

    fn discover(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            id: "mock:0".into(),
            protocol: Protocol::Mock,
            name: "Mock MidiController (dev)".into(),
            image: Protocol::Mock.image_key().into(),
            address: Address::Mock,
            identity: Some(Self::identity()),
        }]
    }

    fn connect(&mut self, _device: &DeviceInfo) -> Result<DeviceIdentity> {
        self.connected = true;
        Ok(Self::identity())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn request(&mut self, req: &Request) -> Result<Response> {
        let resp = match req {
            Request::Identify => Response::ok(json!({
                "name": "Mock MidiController", "firmware": "sim-0.1", "protocol_version": 1,
                "device_id": "MOCK-0001"
            })),
            Request::Ping => Response::empty_ok(),

            Request::ListSets => Response::ok(Self::names(&self.sets)),
            Request::GetSet { name } => Self::get(&self.sets, name, "set"),
            Request::ListSongs => Response::ok(Self::names(&self.songs)),
            Request::GetSong { name } => Self::get(&self.songs, name, "song"),
            Request::ListPedals => Response::ok(Self::names(&self.pedals)),
            Request::GetPedal { name } => Self::get(&self.pedals, name, "pedal"),

            Request::WriteSet { name, data } => {
                self.sets.insert(name.clone(), data.clone());
                Response::empty_ok()
            }
            Request::WriteSong { name, data } => {
                self.songs.insert(name.clone(), data.clone());
                Response::empty_ok()
            }
            Request::WritePart { .. } => Response::empty_ok(),
            Request::WritePedal { name, data } => {
                self.pedals.insert(name.clone(), data.clone());
                Response::empty_ok()
            }

            Request::DeleteSet { name } => {
                self.sets.remove(name);
                Response::empty_ok()
            }
            Request::DeleteSong { name } => {
                self.songs.remove(name);
                Response::empty_ok()
            }
            Request::DeletePart { .. } => Response::empty_ok(),
            Request::DeletePedal { name } => {
                self.pedals.remove(name);
                Response::empty_ok()
            }

            Request::Dpad { direction } => {
                self.display = format!("DPAD {direction} - mock device");
                Response::ok(json!({ "display_message": self.display }))
            }
            Request::Short { button } => {
                self.display = format!("BUTTON {button} SHORT - mock device");
                Response::ok(json!({ "display_message": self.display }))
            }
            Request::Long => {
                self.display = "LONG PRESS - global menu".to_string();
                Response::ok(json!({ "display_message": self.display }))
            }
            Request::ExtraLong => {
                self.display = "XLONG PRESS - power menu".to_string();
                Response::ok(json!({ "display_message": self.display }))
            }
            Request::GetDisplay => {
                Response::ok(json!({ "display_message": self.display }))
            }

            Request::WifiStatus => Response::ok(self.wifi_status()),
            Request::WifiSet { ssid, password: _ } => {
                self.wifi_ssid = ssid.clone();
                self.wifi_enabled = true;
                self.wifi_connected = true;
                self.wifi_ip = "192.168.1.50".to_string();
                Response::ok(self.wifi_status())
            }
            Request::WifiEnable { on } => {
                self.wifi_enabled = *on;
                self.wifi_connected = *on && !self.wifi_ssid.is_empty();
                self.wifi_ip = if self.wifi_connected { "192.168.1.50".to_string() } else { String::new() };
                Response::ok(self.wifi_status())
            }

            // The mock can't reboot; just ack (the real device drops the link).
            Request::Reboot | Request::RebootBootloader => Response::empty_ok(),
        };
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn names(resp: &Response) -> Vec<String> {
        resp.data
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn discover_returns_one_identified_mock_device() {
        let devices = MockTransport::new().discover();
        assert_eq!(devices.len(), 1);
        let d = &devices[0];
        assert_eq!(d.id, "mock:0");
        assert_eq!(d.protocol, Protocol::Mock);
        assert_eq!(d.image, "mock");
        assert!(matches!(d.address, Address::Mock));
        assert_eq!(d.identity.as_ref().unwrap().name, "Mock MidiController");
    }

    #[test]
    fn connect_disconnect_track_state() {
        let mut t = MockTransport::new();
        assert!(!t.is_connected());
        assert_eq!(t.protocol(), Protocol::Mock);

        let id = t.connect(&MockTransport::new().discover()[0]).unwrap();
        assert_eq!(id.firmware, "sim-0.1");
        assert!(t.is_connected());

        t.disconnect().unwrap();
        assert!(!t.is_connected());
    }

    #[test]
    fn identify_and_ping() {
        let mut t = MockTransport::new();
        let id = t.request(&Request::Identify).unwrap();
        assert_eq!(id.data.unwrap()["protocol_version"], 1);
        assert!(t.request(&Request::Ping).unwrap().ok);
    }

    #[test]
    fn lists_contain_seeded_names() {
        let mut t = MockTransport::new();
        assert!(names(&t.request(&Request::ListSets).unwrap()).contains(&"Friday Gig".to_string()));
        assert!(names(&t.request(&Request::ListSongs).unwrap()).contains(&"Intro".to_string()));
        assert!(names(&t.request(&Request::ListPedals).unwrap()).contains(&"Timeline".to_string()));
    }

    #[test]
    fn lists_are_sorted() {
        let mut t = MockTransport::new();
        let got = names(&t.request(&Request::ListSets).unwrap());
        let mut sorted = got.clone();
        sorted.sort();
        assert_eq!(got, sorted);
    }

    #[test]
    fn get_existing_entities() {
        let mut t = MockTransport::new();
        assert_eq!(
            t.request(&Request::GetSet { name: "Friday Gig".into() }).unwrap().data.unwrap()["songs"][0],
            "Intro"
        );
        assert_eq!(
            t.request(&Request::GetSong { name: "Intro".into() }).unwrap().data.unwrap()["tempo"],
            120
        );
        assert!(t.request(&Request::GetPedal { name: "Timeline".into() }).unwrap().ok);
    }

    #[test]
    fn get_missing_entities_error_with_name() {
        let mut t = MockTransport::new();
        for req in [
            Request::GetSet { name: "Nope".into() },
            Request::GetSong { name: "Nope".into() },
            Request::GetPedal { name: "Nope".into() },
        ] {
            let r = t.request(&req).unwrap();
            assert!(!r.ok);
            assert!(r.error.unwrap().contains("Nope"));
        }
    }

    #[test]
    fn write_then_get_roundtrips_each_kind() {
        let mut t = MockTransport::new();

        t.request(&Request::WriteSet { name: "S".into(), data: json!({ "songs": ["x"] }) }).unwrap();
        assert_eq!(t.request(&Request::GetSet { name: "S".into() }).unwrap().data.unwrap()["songs"][0], "x");

        t.request(&Request::WriteSong { name: "So".into(), data: json!({ "tempo": 99 }) }).unwrap();
        assert_eq!(t.request(&Request::GetSong { name: "So".into() }).unwrap().data.unwrap()["tempo"], 99);

        t.request(&Request::WritePedal { name: "Pe".into(), data: json!({ "presets": [7] }) }).unwrap();
        assert_eq!(t.request(&Request::GetPedal { name: "Pe".into() }).unwrap().data.unwrap()["presets"][0], 7);
    }

    #[test]
    fn write_overwrites_existing() {
        let mut t = MockTransport::new();
        t.request(&Request::WriteSet { name: "Friday Gig".into(), data: json!({ "songs": [] }) }).unwrap();
        let after = t.request(&Request::GetSet { name: "Friday Gig".into() }).unwrap();
        assert_eq!(after.data.unwrap()["songs"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn delete_removes_each_kind() {
        let mut t = MockTransport::new();
        t.request(&Request::DeleteSet { name: "Friday Gig".into() }).unwrap();
        assert!(!t.request(&Request::GetSet { name: "Friday Gig".into() }).unwrap().ok);

        t.request(&Request::DeleteSong { name: "Intro".into() }).unwrap();
        assert!(!t.request(&Request::GetSong { name: "Intro".into() }).unwrap().ok);

        t.request(&Request::DeletePedal { name: "Timeline".into() }).unwrap();
        assert!(!t.request(&Request::GetPedal { name: "Timeline".into() }).unwrap().ok);
    }

    #[test]
    fn part_write_and_delete_are_accepted() {
        let mut t = MockTransport::new();
        assert!(t.request(&Request::WritePart { name: "A".into(), data: json!({}) }).unwrap().ok);
        assert!(t.request(&Request::DeletePart { name: "A".into() }).unwrap().ok);
    }

    #[test]
    fn dpad_and_short_return_display_messages() {
        let mut t = MockTransport::new();
        let d = t.request(&Request::Dpad { direction: "up".into() }).unwrap();
        assert!(d.data.unwrap()["display_message"].as_str().unwrap().contains("up"));

        let s = t.request(&Request::Short { button: "2".into() }).unwrap();
        assert!(s.data.unwrap()["display_message"].as_str().unwrap().contains("2"));
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use serde_json::json;

    fn list_len(t: &mut MockTransport, req: Request) -> usize {
        t.request(&req).unwrap().data.unwrap().as_array().unwrap().len()
    }

    #[test]
    fn initial_list_sizes() {
        let mut t = MockTransport::new();
        assert_eq!(list_len(&mut t, Request::ListSets), 2);
        assert_eq!(list_len(&mut t, Request::ListSongs), 3);
        assert_eq!(list_len(&mut t, Request::ListPedals), 3);
    }

    #[test]
    fn all_dpad_directions_echo_in_message() {
        let mut t = MockTransport::new();
        for dir in ["up", "down", "CW", "CCW"] {
            let m = t.request(&Request::Dpad { direction: dir.into() }).unwrap();
            let msg = m.data.unwrap()["display_message"].as_str().unwrap().to_string();
            assert!(msg.contains(dir), "msg {msg} missing {dir}");
        }
    }

    #[test]
    fn all_buttons_echo_in_message() {
        let mut t = MockTransport::new();
        for b in ["1", "2", "3", "4", "5"] {
            let m = t.request(&Request::Short { button: b.into() }).unwrap();
            assert!(m.data.unwrap()["display_message"].as_str().unwrap().contains(b));
        }
    }

    #[test]
    fn writing_grows_each_list() {
        let mut t = MockTransport::new();
        t.request(&Request::WriteSet { name: "Z".into(), data: json!({ "songs": [] }) }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListSets), 3);
        t.request(&Request::WriteSong { name: "Z".into(), data: json!({}) }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListSongs), 4);
        t.request(&Request::WritePedal { name: "Z".into(), data: json!({}) }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListPedals), 4);
    }

    #[test]
    fn deleting_shrinks_each_list() {
        let mut t = MockTransport::new();
        t.request(&Request::DeleteSet { name: "Friday Gig".into() }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListSets), 1);
        t.request(&Request::DeleteSong { name: "Intro".into() }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListSongs), 2);
        t.request(&Request::DeletePedal { name: "Timeline".into() }).unwrap();
        assert_eq!(list_len(&mut t, Request::ListPedals), 2);
    }

    #[test]
    fn deleting_missing_is_still_ok() {
        let mut t = MockTransport::new();
        assert!(t.request(&Request::DeleteSet { name: "ghost".into() }).unwrap().ok);
    }

    #[test]
    fn ping_has_no_data() {
        let mut t = MockTransport::new();
        let r = t.request(&Request::Ping).unwrap();
        assert!(r.ok);
        assert!(r.data.is_none());
    }

    #[test]
    fn identify_reports_protocol_version_one() {
        let mut t = MockTransport::new();
        let id = t.connect(&MockTransport::new().discover()[0]).unwrap();
        assert_eq!(id.protocol_version, 1);
        assert_eq!(id.name, "Mock MidiController");
    }

    #[test]
    fn lists_stay_sorted_after_writes() {
        let mut t = MockTransport::new();
        t.request(&Request::WriteSet { name: "AAA".into(), data: json!({}) }).unwrap();
        t.request(&Request::WriteSet { name: "ZZZ".into(), data: json!({}) }).unwrap();
        let names: Vec<String> = t
            .request(&Request::ListSets)
            .unwrap()
            .data
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert_eq!(names.first().unwrap(), "AAA");
    }
}
