// Thin wrapper over the Tauri dialog plugin, isolated so the UI imports a plain
// function (and tests can mock just this).

import { open } from "@tauri-apps/plugin-dialog";

/** Native open dialog filtered to firmware images. Returns the chosen absolute
 *  path, or null if the user cancelled. */
export async function pickFirmwareFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    title: "Select firmware (.uf2)",
    filters: [{ name: "Firmware", extensions: ["uf2"] }],
  });
  return typeof selected === "string" ? selected : null;
}
