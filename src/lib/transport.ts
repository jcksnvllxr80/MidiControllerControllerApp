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

/** The RP2350 UF2 bootloader drive, when one is mounted. */
export interface BootloaderDrive {
  mount_point: string;
  label: string;
}

/** Look for the RP2350 bootloader drive (present after a reboot_bootloader). */
export function findBootloader(): Promise<BootloaderDrive | null> {
  return invoke<BootloaderDrive | null>("find_bootloader");
}

/** Flash a .uf2 by copying it onto the bootloader drive; returns the dest path. */
export function flashFirmware(uf2Path: string): Promise<string> {
  return invoke<string>("flash_firmware", { uf2Path });
}

export function onDeviceFound(cb: (d: DeviceInfo) => void): Promise<UnlistenFn> {
  return listen<DeviceInfo>("device-found", (e) => cb(e.payload));
}

export function onConnectionStatus(
  cb: (s: ConnectionStatus) => void,
): Promise<UnlistenFn> {
  return listen<ConnectionStatus>("connection-status", (e) => cb(e.payload));
}

export function onDeviceLog(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<string>("device-log", (e) => cb(e.payload));
}

/** Return the directory where app.log and device.log are written. */
export function getLogDir(): Promise<string> {
  return invoke<string>("get_log_dir");
}

/** Send a request and return its data, throwing on transport or logical error. */
export async function request<T = any>(req: Request): Promise<T> {
  const resp = await sendRequest(req);
  if (!resp.ok) {
    throw new Error(resp.error ?? "request failed");
  }
  return resp.data as T;
}
