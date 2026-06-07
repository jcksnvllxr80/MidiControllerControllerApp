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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(AppError::NotConnected.to_string(), "not connected to a device");
        assert_eq!(AppError::Unsupported("Wi-Fi".into()).to_string(), "unsupported transport: Wi-Fi");
        assert_eq!(AppError::Connect("boom".into()).to_string(), "connect failed: boom");
        assert_eq!(AppError::Request("boom".into()).to_string(), "request failed: boom");
        assert_eq!(AppError::Internal("boom".into()).to_string(), "internal error: boom");
    }

    #[test]
    fn serializes_to_plain_string() {
        // The frontend receives the message string, not a tagged object.
        let json = serde_json::to_string(&AppError::NotConnected).unwrap();
        assert_eq!(json, "\"not connected to a device\"");

        let v = serde_json::to_value(AppError::Connect("x".into())).unwrap();
        assert!(v.is_string());
        assert_eq!(v, "connect failed: x");
    }
}
