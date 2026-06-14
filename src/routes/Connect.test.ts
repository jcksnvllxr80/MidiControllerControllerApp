import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({
  scanDevices: vi.fn(),
  connectDevice: vi.fn(),
  onDeviceFound: vi.fn(),
  findBootloader: vi.fn(),
  flashFirmware: vi.fn(),
}));

import * as transport from "../lib/transport";
import Connect from "./Connect.svelte";
import type { DeviceInfo } from "../lib/protocol";

const t = transport as unknown as Record<string, ReturnType<typeof vi.fn>>;

const device: DeviceInfo = {
  id: "mock:0",
  protocol: "mock",
  name: "Mock MidiController (dev)",
  image: "mock",
  address: { kind: "mock" },
  identity: { name: "Mock MidiController", firmware: "sim-0.1", protocol_version: 1 },
};

const serialDevice: DeviceInfo = {
  id: "serial:COM4",
  protocol: "serial",
  name: "COM4 — Pico",
  image: "serial",
  address: { kind: "port", name: "COM4", baud: 115200 },
};

beforeEach(() => {
  t.scanDevices.mockReset().mockResolvedValue([device]);
  t.connectDevice.mockReset().mockResolvedValue({ connected: true });
  t.onDeviceFound.mockReset().mockResolvedValue(() => {});
  t.findBootloader.mockReset().mockResolvedValue(null);
  t.flashFirmware.mockReset().mockResolvedValue("E:\\midicontroller_pico.uf2");
});

describe("Connect screen", () => {
  it("scans on mount and lists discovered devices with name + protocol", async () => {
    render(Connect);
    expect(await screen.findByText("Mock MidiController (dev)")).toBeTruthy();
    expect(screen.getByText("Mock")).toBeTruthy(); // protocol label on the card
    expect(t.scanDevices).toHaveBeenCalled();
  });

  it("renders a card per discovered device", async () => {
    t.scanDevices.mockResolvedValue([device, serialDevice]);
    render(Connect);
    expect(await screen.findByText("Mock MidiController (dev)")).toBeTruthy();
    expect(screen.getByText("COM4 — Pico")).toBeTruthy();
    expect(screen.getByText("Serial")).toBeTruthy();
  });

  it("Connect on a card connects that device", async () => {
    render(Connect);
    const btn = await screen.findByRole("button", { name: /^connect$/i });
    await fireEvent.click(btn);
    expect(t.connectDevice).toHaveBeenCalledWith(device);
  });

  it("re-scans when Scan is clicked", async () => {
    render(Connect);
    await screen.findByText("Mock MidiController (dev)");
    const before = t.scanDevices.mock.calls.length;
    await fireEvent.click(screen.getByRole("button", { name: /scan for devices/i }));
    expect(t.scanDevices.mock.calls.length).toBeGreaterThan(before);
  });

  it("shows an empty state when nothing is found", async () => {
    t.scanDevices.mockResolvedValue([]);
    render(Connect);
    expect(await screen.findByText(/No devices found/i)).toBeTruthy();
  });

  it("surfaces a scan error", async () => {
    t.scanDevices.mockRejectedValue(new Error("usb exploded"));
    render(Connect);
    expect(await screen.findByText(/usb exploded/)).toBeTruthy();
  });

  it("shows the bootloader flash card and flashes a .uf2", async () => {
    t.scanDevices.mockResolvedValue([]);
    t.findBootloader.mockResolvedValue({ mount_point: "E:\\", label: "RP2350" });
    render(Connect);
    expect(await screen.findByText(/RP2350 bootloader/i)).toBeTruthy();
    await fireEvent.input(screen.getByLabelText("Firmware .uf2 path"), {
      target: { value: "C:\\fw\\midicontroller_pico.uf2" },
    });
    await fireEvent.click(screen.getByRole("button", { name: /^flash$/i }));
    expect(t.flashFirmware).toHaveBeenCalledWith("C:\\fw\\midicontroller_pico.uf2");
    expect(await screen.findByText(/will reboot and reconnect/i)).toBeTruthy();
  });
});
