import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the Tauri bridge before importing the module under test.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  scanDevices,
  connectDevice,
  disconnectDevice,
  sendRequest,
  fetchConnectionStatus,
  request,
  onDeviceFound,
  onConnectionStatus,
} from "./transport";
import type { DeviceInfo } from "./protocol";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
const mockListen = listen as unknown as ReturnType<typeof vi.fn>;

const device: DeviceInfo = {
  id: "mock:0",
  protocol: "mock",
  name: "Mock",
  image: "mock",
  address: { kind: "mock" },
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockListen.mockResolvedValue(() => {});
});

describe("transport command wrappers", () => {
  it("scanDevices invokes scan_devices", async () => {
    mockInvoke.mockResolvedValueOnce([device]);
    await expect(scanDevices()).resolves.toEqual([device]);
    expect(mockInvoke).toHaveBeenCalledWith("scan_devices");
  });

  it("connectDevice passes the device argument", async () => {
    mockInvoke.mockResolvedValueOnce({ connected: true });
    await connectDevice(device);
    expect(mockInvoke).toHaveBeenCalledWith("connect_device", { device });
  });

  it("disconnectDevice invokes disconnect_device", async () => {
    mockInvoke.mockResolvedValueOnce({ connected: false });
    await disconnectDevice();
    expect(mockInvoke).toHaveBeenCalledWith("disconnect_device");
  });

  it("sendRequest passes the request argument", async () => {
    mockInvoke.mockResolvedValueOnce({ ok: true });
    await sendRequest({ op: "ping" });
    expect(mockInvoke).toHaveBeenCalledWith("send_request", { request: { op: "ping" } });
  });

  it("fetchConnectionStatus invokes connection_status", async () => {
    mockInvoke.mockResolvedValueOnce({ connected: false });
    await fetchConnectionStatus();
    expect(mockInvoke).toHaveBeenCalledWith("connection_status");
  });
});

describe("request() helper", () => {
  it("returns data when the response is ok", async () => {
    mockInvoke.mockResolvedValueOnce({ ok: true, data: { x: 1 } });
    await expect(request({ op: "get_set", name: "a" })).resolves.toEqual({ x: 1 });
  });

  it("throws the error message when not ok", async () => {
    mockInvoke.mockResolvedValueOnce({ ok: false, error: "no set 'a'" });
    await expect(request({ op: "get_set", name: "a" })).rejects.toThrow("no set 'a'");
  });

  it("throws a default message when error is absent", async () => {
    mockInvoke.mockResolvedValueOnce({ ok: false });
    await expect(request({ op: "ping" })).rejects.toThrow("request failed");
  });
});

describe("event subscriptions", () => {
  it("onDeviceFound subscribes to the device-found event", async () => {
    await onDeviceFound(vi.fn());
    expect(mockListen).toHaveBeenCalledWith("device-found", expect.any(Function));
  });

  it("onConnectionStatus subscribes to the connection-status event", async () => {
    await onConnectionStatus(vi.fn());
    expect(mockListen).toHaveBeenCalledWith("connection-status", expect.any(Function));
  });

  it("forwards the event payload to the callback", async () => {
    let handler: ((e: { payload: unknown }) => void) | undefined;
    mockListen.mockImplementationOnce(
      (_name: string, h: (e: { payload: unknown }) => void) => {
        handler = h;
        return Promise.resolve(() => {});
      },
    );
    const cb = vi.fn();
    await onDeviceFound(cb);
    handler?.({ payload: device });
    expect(cb).toHaveBeenCalledWith(device);
  });
});

describe("request() data handling", () => {
  it("returns undefined data on an ok response with no data", async () => {
    mockInvoke.mockResolvedValueOnce({ ok: true });
    await expect(request({ op: "ping" })).resolves.toBeUndefined();
  });

  it("forwards each call to invoke", async () => {
    mockInvoke.mockResolvedValue({ ok: true, data: 1 });
    await request({ op: "list_sets" });
    await request({ op: "list_songs" });
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });
});

describe("onConnectionStatus payload", () => {
  it("forwards connection-status payloads to the callback", async () => {
    let handler: ((e: { payload: unknown }) => void) | undefined;
    mockListen.mockImplementationOnce(
      (_n: string, h: (e: { payload: unknown }) => void) => {
        handler = h;
        return Promise.resolve(() => {});
      },
    );
    const cb = vi.fn();
    await onConnectionStatus(cb);
    handler?.({ payload: { connected: true } });
    expect(cb).toHaveBeenCalledWith({ connected: true });
  });
});
