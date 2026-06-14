import { writable } from "svelte/store";
import type { ConnectionStatus } from "./protocol";

/** Current connection, kept in sync with backend `connection-status` events. */
export const connection = writable<ConnectionStatus>({ connected: false });

/**
 * Reason the link was lost (heartbeat failure), shown as a banner on Connect.
 * Kept separate from `connection` so the backend's disconnect event can't
 * race-clear the message before the user sees it.
 */
export const connectionError = writable<string>("");

/** Left sidebar collapsed (icons only) vs expanded (icons + labels). Persisted. */
const SIDEBAR_KEY = "sidebarCollapsed";
function loadCollapsed(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_KEY) === "1";
  } catch {
    return false;
  }
}
export const sidebarCollapsed = writable<boolean>(loadCollapsed());
sidebarCollapsed.subscribe((v) => {
  try {
    localStorage.setItem(SIDEBAR_KEY, v ? "1" : "0");
  } catch {
    /* no localStorage (SSR/test) — in-memory only */
  }
});
