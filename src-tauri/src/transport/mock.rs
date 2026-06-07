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

    #[test]
    fn lists_and_gets_seeded_sets() {
        let mut t = MockTransport::new();
        let list = t.request(&Request::ListSets).unwrap();
        let names = list.data.unwrap();
        assert!(names.as_array().unwrap().iter().any(|v| v == "Friday Gig"));

        let got = t.request(&Request::GetSet { name: "Friday Gig".into() }).unwrap();
        assert!(got.ok);
        assert_eq!(got.data.unwrap()["songs"][0], "Intro");
    }

    #[test]
    fn write_then_read_roundtrips() {
        let mut t = MockTransport::new();
        t.request(&Request::WriteSet {
            name: "New".into(),
            data: serde_json::json!({ "name": "New", "songs": [] }),
        })
        .unwrap();
        let got = t.request(&Request::GetSet { name: "New".into() }).unwrap();
        assert!(got.ok);
    }

    #[test]
    fn missing_set_is_an_error_response() {
        let mut t = MockTransport::new();
        let got = t.request(&Request::GetSet { name: "Nope".into() }).unwrap();
        assert!(!got.ok);
        assert!(got.error.unwrap().contains("Nope"));
    }
}
