//! Error type returned from Tauri commands. Serializes to a plain string so the
//! frontend gets a readable message.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not connected to a device")]
    NotConnected,
    #[error("unsupported transport: {0}")]
    Unsupported(String),
    #[error("connect failed: {0}")]
    Connect(String),
    #[error("request failed: {0}")]
    Request(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
