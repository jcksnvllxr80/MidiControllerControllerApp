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
