import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { deviceLogs } from "../lib/stores";

vi.mock("../lib/transport", () => ({
  getLogDir: vi.fn(() => Promise.resolve("")),
}));

import Logs from "./Logs.svelte";

const flush = () => new Promise((r) => setTimeout(r, 10));

beforeEach(() => {
  deviceLogs.set([]);
});

describe("Logs view", () => {
  it("shows empty state initially", async () => {
    render(Logs);
    expect(screen.getByText(/no log output yet/i)).toBeTruthy();
    expect(screen.getByText(/0 lines/i)).toBeTruthy();
  });

  it("displays log lines as they arrive", async () => {
    render(Logs);
    deviceLogs.update((l) => [...l, "boot: system ready", "midi: note on ch=1"]);
    await flush();
    expect(screen.getByText("boot: system ready")).toBeTruthy();
    expect(screen.getByText("midi: note on ch=1")).toBeTruthy();
    expect(screen.getByText(/2 lines/i)).toBeTruthy();
  });

  it("clear button removes all lines", async () => {
    render(Logs);
    deviceLogs.update((l) => [...l, "some log"]);
    await flush();
    expect(screen.getByText("some log")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /clear/i }));
    await flush();
    expect(screen.getByText(/no log output yet/i)).toBeTruthy();
    expect(screen.getByText(/0 lines/i)).toBeTruthy();
  });
});
