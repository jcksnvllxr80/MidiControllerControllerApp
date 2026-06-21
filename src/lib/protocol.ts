// TypeScript mirror of the Rust protocol + transport types. Keeping these in
// sync with src-tauri makes the wire contract compile-checked on this side.

export type Protocol = "serial" | "usb" | "wifi" | "ethernet" | "mock";

export interface DeviceIdentity {
  name: string;
  firmware: string;
  protocol_version: number;
  /** Stable per-unit id (same over USB and WiFi); absent on older firmware. */
  device_id?: string;
}

export type Address =
  | { kind: "port"; name: string; baud: number }
  | { kind: "usb"; vid: number; pid: number; serial?: string }
  | { kind: "net"; host: string; port: number }
  | { kind: "mock" };

export interface DeviceInfo {
  id: string;
  protocol: Protocol;
  name: string;
  image: string;
  address: Address;
  identity?: DeviceIdentity;
}

export interface ConnectionStatus {
  connected: boolean;
  device?: DeviceInfo;
  identity?: DeviceIdentity;
  error?: string;
}

export interface ProtoResponse {
  ok: boolean;
  data?: unknown;
  error?: string;
}

// Requests are op-tagged objects matching the Rust `Request` enum.
export type Request =
  | { op: "identify" }
  | { op: "ping" }
  | { op: "list_sets" }
  | { op: "get_set"; name: string }
  | { op: "list_songs" }
  | { op: "get_song"; name: string }
  | { op: "list_pedals" }
  | { op: "get_pedal"; name: string }
  | { op: "write_set"; name: string; data: unknown }
  | { op: "write_song"; name: string; data: unknown }
  | { op: "write_part"; name: string; data: unknown }
  | { op: "write_pedal"; name: string; data: unknown }
  | { op: "delete_set"; name: string }
  | { op: "delete_song"; name: string }
  | { op: "delete_part"; name: string }
  | { op: "delete_pedal"; name: string }
  | { op: "dpad"; direction: string }
  | { op: "short"; button: string }
  | { op: "long" }
  | { op: "extra_long" }
  | { op: "get_display" }
  | { op: "wifi_status" }
  | { op: "wifi_set"; ssid: string; password?: string }
  | { op: "wifi_enable"; on: boolean }
  | { op: "reboot" }
  | { op: "reboot_bootloader" };

/** `data` returned by the wifi_* ops. */
export interface WifiStatus {
  enabled: boolean;
  connected: boolean;
  ssid: string;
  ip: string;
}

// The three config entity kinds, and their op names, for the Configure manager.
export type EntityKind = "set" | "song" | "pedal";

export const ENTITY_OPS: Record<
  EntityKind,
  { list: Request["op"]; get: Request["op"]; write: Request["op"]; del: Request["op"] }
> = {
  set: { list: "list_sets", get: "get_set", write: "write_set", del: "delete_set" },
  song: { list: "list_songs", get: "get_song", write: "write_song", del: "delete_song" },
  pedal: { list: "list_pedals", get: "get_pedal", write: "write_pedal", del: "delete_pedal" },
};

export const PROTOCOL_LABEL: Record<Protocol, string> = {
  serial: "Serial",
  usb: "USB",
  wifi: "Wi-Fi",
  ethernet: "Ethernet",
  mock: "Mock",
};
