import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("./lib/transport", () => ({
  scanDevices: vi.fn(() => Promise.resolve([])),
  connectDevice: vi.fn(() => Promise.resolve({ connected: true })),
  disconnectDevice: vi.fn(() => Promise.resolve({ connected: false })),
  onDeviceFound: vi.fn(() => Promise.resolve(() => {})),
  onConnectionStatus: vi.fn(() => Promise.resolve(() => {})),
  fetchConnectionStatus: vi.fn(() => Promise.resolve({ connected: false })),
  sendRequest: vi.fn(() => Promise.resolve({ ok: true })),
  request: vi.fn(() => Promise.resolve(null)),
}));
vi.mock("./lib/dialog", () => ({ pickFirmwareFile: vi.fn(() => Promise.resolve(null)) }));

import * as transport from "./lib/transport";
import { connection } from "./lib/stores";
import App from "./App.svelte";
import type { ConnectionStatus } from "./lib/protocol";

const t = transport as unknown as Record<string, ReturnType<typeof vi.fn>>;

const connected: ConnectionStatus = {
  connected: true,
  device: { id: "mock:0", protocol: "mock", name: "Mock", image: "mock", address: { kind: "mock" } },
  identity: { name: "Mock MidiController", firmware: "sim-0.1", protocol_version: 1 },
};

beforeEach(() => {
  connection.set({ connected: false });
  t.scanDevices.mockReset().mockResolvedValue([]);
  t.onDeviceFound.mockReset().mockResolvedValue(() => {});
  t.onConnectionStatus.mockReset().mockResolvedValue(() => {});
  t.fetchConnectionStatus.mockReset().mockResolvedValue({ connected: false });
  t.disconnectDevice.mockReset().mockResolvedValue({ connected: false });
  t.request.mockReset().mockResolvedValue(null);
});

describe("App shell", () => {
  it("shows the Connect screen when disconnected", async () => {
    render(App);
    expect(await screen.findByRole("button", { name: /scan for devices/i })).toBeTruthy();
  });

  it("shows the sidebar nav + disconnect when connected", async () => {
    t.fetchConnectionStatus.mockResolvedValue(connected);
    render(App);
    expect(await screen.findByRole("button", { name: "Disconnect" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Configure" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "JSON" })).toBeTruthy();
  });

  it("defaults to the Control view when connected", async () => {
    t.fetchConnectionStatus.mockResolvedValue(connected);
    render(App);
    // the Select control only exists in the Control view
    expect(await screen.findByRole("button", { name: "Select" })).toBeTruthy();
  });

  it("nav switches to the Configure view", async () => {
    t.fetchConnectionStatus.mockResolvedValue(connected);
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "Configure" }));
    // Configure shows entity tabs
    expect(await screen.findByRole("button", { name: "Sets" })).toBeTruthy();
  });

  it("Disconnect calls disconnectDevice", async () => {
    t.fetchConnectionStatus.mockResolvedValue(connected);
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "Disconnect" }));
    expect(t.disconnectDevice).toHaveBeenCalled();
  });

  it("reacts to a connection-status event flipping to connected", async () => {
    let push: ((s: ConnectionStatus) => void) | undefined;
    t.onConnectionStatus.mockImplementation((cb: (s: ConnectionStatus) => void) => {
      push = cb;
      return Promise.resolve(() => {});
    });
    render(App);
    await screen.findByRole("button", { name: /scan for devices/i });
    push?.(connected);
    expect(await screen.findByRole("button", { name: "Disconnect" })).toBeTruthy();
  });
});
