// Thin typed wrapper over the Tauri command + event bridge. The rest of the UI
// talks to the controller exclusively through this module — it never knows
// which link family (serial / usb / ...) is actually in use.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConnectionStatus,
  DeviceInfo,
  ProtoResponse,
  Request,
} from "./protocol";

export function scanDevices(): Promise<DeviceInfo[]> {
  return invoke<DeviceInfo[]>("scan_devices");
}

export function connectDevice(device: DeviceInfo): Promise<ConnectionStatus> {
  return invoke<ConnectionStatus>("connect_device", { device });
}

export function disconnectDevice(): Promise<ConnectionStatus> {
  return invoke<ConnectionStatus>("disconnect_device");
}

export function sendRequest(request: Request): Promise<ProtoResponse> {
  return invoke<ProtoResponse>("send_request", { request });
}

export function fetchConnectionStatus(): Promise<ConnectionStatus> {
  return invoke<ConnectionStatus>("connection_status");
}

export function onDeviceFound(cb: (d: DeviceInfo) => void): Promise<UnlistenFn> {
  return listen<DeviceInfo>("device-found", (e) => cb(e.payload));
}

export function onConnectionStatus(
  cb: (s: ConnectionStatus) => void,
): Promise<UnlistenFn> {
  return listen<ConnectionStatus>("connection-status", (e) => cb(e.payload));
}

/** Send a request and return its data, throwing on transport or logical error. */
export async function request<T = any>(req: Request): Promise<T> {
  const resp = await sendRequest(req);
  if (!resp.ok) {
    throw new Error(resp.error ?? "request failed");
  }
  return resp.data as T;
}
