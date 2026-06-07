import { writable } from "svelte/store";
import type { ConnectionStatus } from "./protocol";

/** Current connection, kept in sync with backend `connection-status` events. */
export const connection = writable<ConnectionStatus>({ connected: false });
