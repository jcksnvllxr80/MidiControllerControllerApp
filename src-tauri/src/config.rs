//! Optional `config.toml` read from the directory containing the executable.
//!
//! Every field has a sensible default; a missing or malformed file is silently
//! ignored so start-up never fails because of a config problem.
//!
//! ## Minimal example
//! ```toml
//! # config.toml — place next to the application executable
//! # Override the directory where app.log and device.log are written.
//! # Defaults to: {exe_directory}/logs
//! # log_path = "C:/Users/Me/Documents/midi-logs"
//! ```

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    /// Directory for `app.log` and `device.log`.
    /// Defaults to a `logs` folder next to the application executable.
    pub log_path: Option<PathBuf>,
}

impl AppConfig {
    /// Load `config.toml` from the directory that contains the running
    /// executable. Returns an all-defaults config if the file is absent,
    /// unreadable, or cannot be parsed — never panics or returns an error.
    pub fn load() -> Self {
        let text = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join("config.toml")))
            .and_then(|path| std::fs::read_to_string(path).ok());

        match text {
            Some(t) => toml::from_str(&t).unwrap_or_default(),
            None => Self::default(),
        }
    }
}
