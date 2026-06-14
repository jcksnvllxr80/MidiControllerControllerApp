import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({
  request: vi.fn(() => Promise.resolve(null)),
  disconnectDevice: vi.fn(() => Promise.resolve({ connected: false })),
}));

import * as transport from "../lib/transport";
import { connection } from "../lib/stores";
import Firmware from "./Firmware.svelte";

const t = transport as unknown as Record<string, ReturnType<typeof vi.fn>>;
const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  t.request.mockReset().mockResolvedValue(null);
  t.disconnectDevice.mockReset().mockResolvedValue({ connected: false });
  connection.set({
    connected: true,
    device: { id: "serial:COM4", protocol: "serial", name: "MidiController", image: "serial", address: { kind: "port", name: "COM4", baud: 115200 } },
    identity: { name: "MidiController", firmware: "1.4.2", protocol_version: 1, device_id: "E66-0042" },
  });
});

describe("Firmware screen", () => {
  it("shows the firmware version and device id", async () => {
    render(Firmware);
    expect(await screen.findByText(/v1\.4\.2/)).toBeTruthy();
    expect(screen.getByText(/E66-0042/)).toBeTruthy();
  });

  it("Reboot sends op:reboot then disconnects locally", async () => {
    render(Firmware);
    await fireEvent.click(screen.getByRole("button", { name: /^reboot$/i }));
    await flush();
    expect(t.request).toHaveBeenCalledWith({ op: "reboot" });
    expect(t.disconnectDevice).toHaveBeenCalled();
  });

  it("Update firmware sends op:reboot_bootloader then disconnects locally", async () => {
    render(Firmware);
    await fireEvent.click(screen.getByRole("button", { name: /update firmware/i }));
    await flush();
    expect(t.request).toHaveBeenCalledWith({ op: "reboot_bootloader" });
    expect(t.disconnectDevice).toHaveBeenCalled();
  });
});
