// Window-control wrappers for the custom (decorations:false) title bar. Each is
// guarded so the app still renders outside a Tauri window (vitest / vite
// preview), where `getCurrentWindow()` has no backend — the buttons just no-op.

import { getCurrentWindow } from "@tauri-apps/api/window";

function win() {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export async function minimize(): Promise<void> {
  try {
    await win()?.minimize();
  } catch {
    /* not in a Tauri window */
  }
}

export async function toggleMaximize(): Promise<void> {
  try {
    await win()?.toggleMaximize();
  } catch {
    /* not in a Tauri window */
  }
}

export async function closeWindow(): Promise<void> {
  try {
    await win()?.close();
  } catch {
    /* not in a Tauri window */
  }
}

export async function isMaximized(): Promise<boolean> {
  try {
    return (await win()?.isMaximized()) ?? false;
  } catch {
    return false;
  }
}

/** Subscribe to resize so the maximize/restore icon stays in sync. Returns an
 *  unlisten fn (a no-op outside Tauri). */
export async function onResized(cb: () => void): Promise<() => void> {
  try {
    const un = await win()?.onResized(() => cb());
    return un ?? (() => {});
  } catch {
    return () => {};
  }
}
