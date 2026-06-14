import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({ request: vi.fn() }));

import * as transport from "../lib/transport";
import Wifi from "./Wifi.svelte";

const t = transport as unknown as { request: ReturnType<typeof vi.fn> };

const status = (over: Partial<Record<string, unknown>> = {}) => ({
  enabled: false,
  connected: false,
  ssid: "",
  ip: "",
  ...over,
});

beforeEach(() => {
  t.request.mockReset().mockResolvedValue(status());
});

describe("Wifi screen", () => {
  it("queries wifi_status on mount", async () => {
    render(Wifi);
    await screen.findByPlaceholderText("MyNetwork");
    expect(t.request).toHaveBeenCalledWith({ op: "wifi_status" });
  });

  it("prefills the SSID and shows the connected IP from status", async () => {
    t.request.mockResolvedValue(status({ enabled: true, connected: true, ssid: "Studio", ip: "192.168.1.50" }));
    render(Wifi);
    expect(await screen.findByDisplayValue("Studio")).toBeTruthy();
    expect(screen.getByText(/192\.168\.1\.50/)).toBeTruthy();
  });

  it("saving sends wifi_set with ssid + password and reports the IP", async () => {
    render(Wifi);
    await fireEvent.input(await screen.findByPlaceholderText("MyNetwork"), { target: { value: "Studio" } });
    await fireEvent.input(screen.getByPlaceholderText(/open network/i), { target: { value: "hunter2" } });
    t.request.mockResolvedValue(status({ enabled: true, connected: true, ssid: "Studio", ip: "10.0.0.9" }));
    await fireEvent.click(screen.getByRole("button", { name: /save & connect/i }));
    expect(t.request).toHaveBeenCalledWith({ op: "wifi_set", ssid: "Studio", password: "hunter2" });
    // The IP shows in both the success note and the status line.
    expect((await screen.findAllByText(/10\.0\.0\.9/)).length).toBeGreaterThan(0);
  });

  it("omits password for an open network", async () => {
    render(Wifi);
    await fireEvent.input(await screen.findByPlaceholderText("MyNetwork"), { target: { value: "OpenNet" } });
    await fireEvent.click(screen.getByRole("button", { name: /save & connect/i }));
    expect(t.request).toHaveBeenCalledWith({ op: "wifi_set", ssid: "OpenNet" });
  });

  it("toggles wifi_enable", async () => {
    t.request.mockResolvedValue(status({ enabled: false }));
    render(Wifi);
    const toggle = await screen.findByRole("button", { name: /wi-fi off/i });
    t.request.mockResolvedValue(status({ enabled: true }));
    await fireEvent.click(toggle);
    expect(t.request).toHaveBeenCalledWith({ op: "wifi_enable", on: true });
  });
});
