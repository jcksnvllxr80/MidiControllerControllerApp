import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({ request: vi.fn() }));

import * as transport from "../lib/transport";
import Configure from "./Configure.svelte";

const t = transport as unknown as { request: ReturnType<typeof vi.fn> };

function routeRequest(req: any): Promise<unknown> {
  switch (req.op) {
    case "list_sets":
      return Promise.resolve(["Friday Gig", "Acoustic Set"]);
    case "list_songs":
      return Promise.resolve(["Intro"]);
    case "list_pedals":
      return Promise.resolve(["Timeline"]);
    case "get_set":
      return Promise.resolve({ name: req.name, songs: ["Intro"] });
    case "get_song":
      return Promise.resolve({ name: req.name, tempo: 120, parts: {} });
    default:
      return Promise.resolve(null);
  }
}

beforeEach(() => {
  t.request.mockReset().mockImplementation(routeRequest as any);
});

describe("Configure screen", () => {
  it("loads the set list on mount", async () => {
    render(Configure);
    expect(await screen.findByText("Friday Gig")).toBeTruthy();
    expect(screen.getByText("Acoustic Set")).toBeTruthy();
  });

  it("selecting an item loads it into the editor", async () => {
    render(Configure);
    await fireEvent.click(await screen.findByText("Friday Gig"));
    expect(await screen.findByDisplayValue("Friday Gig")).toBeTruthy();
    expect(t.request).toHaveBeenCalledWith({ op: "get_set", name: "Friday Gig" });
  });

  it("New creates a template and Save writes it", async () => {
    render(Configure);
    await screen.findByText("Friday Gig");
    await fireEvent.click(screen.getByRole("button", { name: /new set/i }));
    await fireEvent.click(screen.getByRole("button", { name: /save set/i }));
    const ops = t.request.mock.calls.map((c) => (c[0] as any).op);
    expect(ops).toContain("write_set");
  });

  it("deleting an item issues a delete op", async () => {
    render(Configure);
    await screen.findByText("Friday Gig");
    const trash = screen.getAllByTitle("Delete")[0];
    await fireEvent.click(trash);
    const ops = t.request.mock.calls.map((c) => (c[0] as any).op);
    expect(ops).toContain("delete_set");
  });

  it("the icon rail switches between sets, songs and pedals", async () => {
    render(Configure);
    await screen.findByText("Friday Gig");
    await fireEvent.click(screen.getByRole("button", { name: "Songs" }));
    expect(await screen.findByText("Intro")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Pedals" }));
    expect(await screen.findByText("Timeline")).toBeTruthy();
  });

  it("the set editor adds a song via the dropdown (no raw JSON)", async () => {
    render(Configure);
    await fireEvent.click(await screen.findByText("Friday Gig"));
    // The set already contains "Intro" as a chip in the editor.
    expect(await screen.findByDisplayValue("Friday Gig")).toBeTruthy();
    expect(screen.getByText("Songs in this set · 1")).toBeTruthy();
  });

  it("Advanced raw JSON rejects invalid input", async () => {
    render(Configure);
    await screen.findByText("Friday Gig");
    await fireEvent.click(screen.getByRole("button", { name: /new set/i }));
    await fireEvent.click(screen.getByRole("button", { name: /advanced.*raw json/i }));
    const raw = screen.getByLabelText("Raw JSON");
    await fireEvent.input(raw, { target: { value: "{ not json" } });
    await fireEvent.click(screen.getByRole("button", { name: /apply json/i }));
    expect(await screen.findByText(/Invalid JSON/i)).toBeTruthy();
  });
});
