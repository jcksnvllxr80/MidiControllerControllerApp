//! The wire protocol shared with the firmware's `IConfigTransport`. One framed
//! request/response channel replaces the web app's two HTTP services (config +
//! control). `op` is the discriminant on the wire; `codec` handles framing.

use serde::{Deserialize, Serialize};

pub mod codec;

/// A request to the controller firmware.
///
/// Internally tagged by `op`, e.g. `{"op":"get_set","name":"Friday Gig"}`.
/// The `codec` adds a correlation `id` when framing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Identify handshake — used by `connect` to confirm a device is ours.
    Identify,
    /// Lightweight liveness check for the heartbeat poll.
    Ping,

    ListSets,
    GetSet { name: String },
    ListSongs,
    GetSong { name: String },
    ListPedals,
    GetPedal { name: String },

    WriteSet { name: String, data: serde_json::Value },
    WriteSong { name: String, data: serde_json::Value },
    WritePart { name: String, data: serde_json::Value },
    WritePedal { name: String, data: serde_json::Value },

    DeleteSet { name: String },
    DeleteSong { name: String },
    DeletePart { name: String },
    DeletePedal { name: String },

    /// Live control: d-pad direction (`up`/`down`/`CW`/`CCW`).
    Dpad { direction: String },
    /// Live control: short button press (`1`..`5`).
    Short { button: String },
}

/// The firmware's reply. `data` is op-specific JSON; on failure `error` is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: serde_json::Value) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }
    pub fn empty_ok() -> Self {
        Self { ok: true, data: None, error: None }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn op_of(req: &Request) -> String {
        serde_json::to_value(req).unwrap()["op"].as_str().unwrap().to_string()
    }

    #[test]
    fn every_variant_serializes_to_its_locked_op_string() {
        // This is the wire contract with the firmware's IConfigTransport. The TS
        // side (src/lib/protocol.ts) must match these exact strings.
        let cases: Vec<(Request, &str)> = vec![
            (Request::Identify, "identify"),
            (Request::Ping, "ping"),
            (Request::ListSets, "list_sets"),
            (Request::GetSet { name: "a".into() }, "get_set"),
            (Request::ListSongs, "list_songs"),
            (Request::GetSong { name: "a".into() }, "get_song"),
            (Request::ListPedals, "list_pedals"),
            (Request::GetPedal { name: "a".into() }, "get_pedal"),
            (Request::WriteSet { name: "a".into(), data: json!({}) }, "write_set"),
            (Request::WriteSong { name: "a".into(), data: json!({}) }, "write_song"),
            (Request::WritePart { name: "a".into(), data: json!({}) }, "write_part"),
            (Request::WritePedal { name: "a".into(), data: json!({}) }, "write_pedal"),
            (Request::DeleteSet { name: "a".into() }, "delete_set"),
            (Request::DeleteSong { name: "a".into() }, "delete_song"),
            (Request::DeletePart { name: "a".into() }, "delete_part"),
            (Request::DeletePedal { name: "a".into() }, "delete_pedal"),
            (Request::Dpad { direction: "up".into() }, "dpad"),
            (Request::Short { button: "1".into() }, "short"),
        ];
        assert_eq!(cases.len(), 18, "every Request variant must be covered");
        for (req, expected) in &cases {
            assert_eq!(&op_of(req), expected);
        }
    }

    #[test]
    fn struct_variants_carry_their_fields() {
        let v = serde_json::to_value(Request::GetSong { name: "Intro".into() }).unwrap();
        assert_eq!(v["name"], "Intro");

        let v = serde_json::to_value(Request::Dpad { direction: "CW".into() }).unwrap();
        assert_eq!(v["direction"], "CW");

        let v = serde_json::to_value(Request::Short { button: "3".into() }).unwrap();
        assert_eq!(v["button"], "3");
    }

    #[test]
    fn request_deserializes_from_wire_json() {
        let r: Request = serde_json::from_str(r#"{"op":"get_set","name":"Gig"}"#).unwrap();
        assert!(matches!(r, Request::GetSet { name } if name == "Gig"));

        let r: Request = serde_json::from_str(r#"{"op":"identify"}"#).unwrap();
        assert!(matches!(r, Request::Identify));

        let r: Request =
            serde_json::from_str(r#"{"op":"write_pedal","name":"P","data":{"x":1}}"#).unwrap();
        assert!(matches!(r, Request::WritePedal { .. }));
    }

    #[test]
    fn unknown_op_fails_to_deserialize() {
        assert!(serde_json::from_str::<Request>(r#"{"op":"frobnicate"}"#).is_err());
    }

    #[test]
    fn response_constructors() {
        assert!(Response::ok(json!(1)).ok);
        assert!(Response::empty_ok().ok);
        let e = Response::err("bad");
        assert!(!e.ok);
        assert_eq!(e.error.unwrap(), "bad");
    }

    #[test]
    fn ok_response_omits_error_field() {
        let v = serde_json::to_value(Response::ok(json!({"a":1}))).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["data"]["a"], 1);
        assert!(v.get("error").is_none());
    }

    #[test]
    fn empty_ok_omits_data_and_error() {
        let v = serde_json::to_value(Response::empty_ok()).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v.get("data").is_none());
        assert!(v.get("error").is_none());
    }

    #[test]
    fn response_deserializes_ignoring_correlation_id() {
        let r: Response = serde_json::from_str(r#"{"id":9,"ok":true,"data":[1,2]}"#).unwrap();
        assert!(r.ok);
        assert_eq!(r.data.unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn response_defaults_ok_false_when_missing() {
        let r: Response = serde_json::from_str(r#"{"error":"x"}"#).unwrap();
        assert!(!r.ok);
        assert_eq!(r.error.unwrap(), "x");
    }
}
