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
}

impl MockTransport {
    pub fn new() -> Self {
        let mut t = Self {
            connected: false,
            sets: BTreeMap::new(),
            songs: BTreeMap::new(),
            pedals: BTreeMap::new(),
        };
        t.seed();
        t
    }

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            name: "Mock MidiController".to_string(),
            firmware: "sim-0.1".to_string(),
            protocol_version: 1,
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
                "name": "Mock MidiController", "firmware": "sim-0.1", "protocol_version": 1
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
                Response::ok(json!({ "display_message": format!("DPAD {direction} - mock device") }))
            }
            Request::Short { button } => {
                Response::ok(json!({ "display_message": format!("BUTTON {button} - mock device") }))
            }
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
