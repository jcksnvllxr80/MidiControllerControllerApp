//! Firmware-update helpers: locate the RP2350 UF2 bootloader drive and copy a
//! `.uf2` onto it (a plain file copy is what flashes the board). The bootloader
//! mounts as a FAT volume labelled `RP2350` (VID 0x2E8A / PID 0x000F).
//!
//! Hardware-dependent and **unverified here** (no board): the drive is matched
//! by its volume label via `sysinfo`, and flashing is a `std::fs::copy`.

use std::path::Path;

use serde::Serialize;
use sysinfo::Disks;

/// Volume label the RP2350 UF2 bootloader presents (RP2040 uses `RPI-RP2`).
const BOOTLOADER_LABEL: &str = "RP2350";

#[derive(Debug, Clone, Serialize)]
pub struct BootloaderDrive {
    /// Mount point / drive root, e.g. `E:\` on Windows or `/Volumes/RP2350`.
    pub mount_point: String,
    pub label: String,
}

/// Scan mounted volumes for the RP2350 bootloader drive (by volume label).
pub fn find_bootloader() -> Option<BootloaderDrive> {
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let label = disk.name().to_string_lossy().to_string();
        if label.eq_ignore_ascii_case(BOOTLOADER_LABEL) {
            return Some(BootloaderDrive {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                label,
            });
        }
    }
    None
}

/// Copy a `.uf2` onto the bootloader drive; the board flashes it and reboots.
/// Returns the destination path. Pure file I/O so it's unit-testable.
pub fn flash_uf2(uf2_path: &str, mount_point: &str) -> Result<String, String> {
    let src = Path::new(uf2_path);
    let is_uf2 = src
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("uf2"));
    if !is_uf2 {
        return Err("Choose a .uf2 file (e.g. midicontroller_pico.uf2).".into());
    }
    if !src.is_file() {
        return Err(format!("File not found: {uf2_path}"));
    }
    let file_name = src.file_name().ok_or("invalid source path")?;
    let dest = Path::new(mount_point).join(file_name);
    std::fs::copy(src, &dest).map_err(|e| format!("copy to {} failed: {e}", dest.display()))?;
    Ok(dest.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_uf2() {
        assert!(flash_uf2("firmware.bin", ".").is_err());
        assert!(flash_uf2("noext", ".").is_err());
    }

    #[test]
    fn rejects_missing_file() {
        let err = flash_uf2("definitely-not-here.uf2", ".").unwrap_err();
        assert!(err.contains("not found"), "got: {err}");
    }

    #[test]
    fn find_bootloader_does_not_panic() {
        // No board attached in CI: just must not panic and return None/Some cleanly.
        let _ = find_bootloader();
    }
}
