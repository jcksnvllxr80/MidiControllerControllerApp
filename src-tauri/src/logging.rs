//! Process-wide logging initialisation (via the `log` façade, routed by fern).
//!
//! Two append-only files are created/opened in `log_dir`:
//!
//! - **`app.log`** — application events at INFO and above from all modules
//!   *except* the `"device"` target. Format: `[timestamp][LEVEL] message`.
//!
//! - **`device.log`** — raw firmware serial-output lines, emitted by the
//!   serial transport with `log::info!(target: "device", "{}", line)`.
//!   Format: `[timestamp] message` (no level prefix — the source is always the
//!   firmware).
//!
//! Both files are **appended** across restarts so you can correlate events from
//! multiple sessions. Delete them manually when rotation is desired.
//!
//! Call `init()` once at startup from `lib.rs::run()`. Calling it a second
//! time (or calling it when another logger is already set) returns `Ok(())` as
//! a no-op — the first one wins. `log::*` macros used before `init()` is called
//! (including in test contexts) are silently discarded by the `log` crate.

use std::path::Path;

use anyhow::{anyhow, Result};

/// Installs the file-backed logger.  Creates `log_dir` if it doesn't exist.
pub fn init(log_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(log_dir)
        .map_err(|e| anyhow!("cannot create log directory {}: {e}", log_dir.display()))?;

    let app_log = log_dir.join("app.log");
    let dev_log = log_dir.join("device.log");

    let result = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        // ── app.log (all targets except "device") ───────────────────────────
        .chain(
            fern::Dispatch::new()
                .filter(|m| m.target() != "device")
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level(),
                        message
                    ))
                })
                .chain(
                    fern::log_file(&app_log)
                        .map_err(|e| anyhow!("cannot open {}: {e}", app_log.display()))?,
                ),
        )
        // ── device.log (only target == "device") ────────────────────────────
        .chain(
            fern::Dispatch::new()
                .filter(|m| m.target() == "device")
                .format(|out, message, _record| {
                    out.finish(format_args!(
                        "[{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        message
                    ))
                })
                .chain(
                    fern::log_file(&dev_log)
                        .map_err(|e| anyhow!("cannot open {}: {e}", dev_log.display()))?,
                ),
        )
        .apply();

    // apply() only fails with SetLoggerError when a logger is already set.
    // Treat that as a harmless no-op — the first installer wins.
    let _ = result;
    Ok(())
}
