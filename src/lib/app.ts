// App-level Tauri helpers. Guarded so they no-op outside a Tauri window
// (vitest / vite preview) instead of throwing.

import { getVersion } from "@tauri-apps/api/app";

/** The app version (from tauri.conf.json → package.json). "" outside Tauri. */
export async function appVersion(): Promise<string> {
  try {
    return (await getVersion()) ?? "";
  } catch {
    return "";
  }
}
