import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({ request: vi.fn() }));

import * as transport from "../lib/transport";
import Control from "./Control.svelte";

const t = transport as unknown as { request: ReturnType<typeof vi.fn> };

beforeEach(() => {
  t.request.mockReset().mockResolvedValue({ display_message: "OK" });
});

describe("Control screen", () => {
  it("d-pad up sends a dpad/up request", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "↑" }));
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "up" });
  });

  it("d-pad left/right map to CCW/CW", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "←" }));
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "CCW" });
    await fireEvent.click(screen.getByRole("button", { name: "→" }));
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "CW" });
  });

  it("d-pad down sends dpad/down", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "↓" }));
    expect(t.request).toHaveBeenCalledWith({ op: "dpad", direction: "down" });
  });

  it("Select button sends short/2", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "Select" }));
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "2" });
  });

  it("Song up/down map to buttons 5/4", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "Song ↑" }));
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "5" });
    await fireEvent.click(screen.getByRole("button", { name: "Song ↓" }));
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "4" });
  });

  it("Part up/down map to buttons 3/1", async () => {
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "Part ↑" }));
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "3" });
    await fireEvent.click(screen.getByRole("button", { name: "Part ↓" }));
    expect(t.request).toHaveBeenCalledWith({ op: "short", button: "1" });
  });

  it("renders the device display message, newlines from ' - '", async () => {
    t.request.mockResolvedValue({ display_message: "PART 1 - SONG A" });
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "↓" }));
    // findByDisplayValue normalizes whitespace, so the newline shows as a space.
    const display = (await screen.findByDisplayValue(/PART 1\s+SONG A/)) as HTMLTextAreaElement;
    expect(display.value).toBe("PART 1\nSONG A");
  });

  it("shows an error when a request rejects", async () => {
    t.request.mockRejectedValue(new Error("boom"));
    render(Control);
    await fireEvent.click(screen.getByRole("button", { name: "↑" }));
    expect(await screen.findByText(/boom/)).toBeTruthy();
  });
});
