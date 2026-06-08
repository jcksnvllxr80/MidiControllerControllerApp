import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({ request: vi.fn() }));

import * as transport from "../lib/transport";
import Control from "./Control.svelte";

const t = transport as unknown as { request: ReturnType<typeof vi.fn> };

function click(name: string) {
  return fireEvent.click(screen.getByRole("button", { name }));
}

beforeEach(() => {
  t.request.mockReset().mockResolvedValue({ display_message: "OK" });
});

describe("Control surface", () => {
  it("shows the idle readout before any input", () => {
    render(Control);
    expect(screen.getByText("Ready")).toBeTruthy();
  });

  it("song next/prev map to buttons 5/4", async () => {
    render(Control);
    await click("Next song");
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "5" });
    await click("Previous song");
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "4" });
  });

  it("part next/prev map to buttons 3/1", async () => {
    render(Control);
    await click("Next part");
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "3" });
    await click("Previous part");
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "1" });
  });

  it("Select sends short/2", async () => {
    render(Control);
    await click("Select");
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "2" });
  });

  it("menu up/down send dpad up/down", async () => {
    render(Control);
    await click("Menu up");
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "up" });
    await click("Menu down");
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "down" });
  });

  it("rotate maps to dpad CCW/CW", async () => {
    render(Control);
    await click("Rotate counter-clockwise");
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "CCW" });
    await click("Rotate clockwise");
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "CW" });
  });

  it("renders the device readout, one line per ' - ' segment", async () => {
    t.request.mockResolvedValue({ display_message: "PART 1 - SONG A" });
    render(Control);
    await click("Select");
    expect(await screen.findByText("PART 1")).toBeTruthy();
    expect(screen.getByText("SONG A")).toBeTruthy();
  });

  it("shows an error when a request rejects", async () => {
    t.request.mockRejectedValue(new Error("boom"));
    render(Control);
    await click("Select");
    expect(await screen.findByText(/boom/)).toBeTruthy();
  });
});
