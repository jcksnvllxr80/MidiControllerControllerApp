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
