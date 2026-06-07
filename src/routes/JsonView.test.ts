import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

vi.mock("../lib/transport", () => ({ request: vi.fn() }));

import * as transport from "../lib/transport";
import JsonView from "./JsonView.svelte";

const t = transport as unknown as { request: ReturnType<typeof vi.fn> };

function routeRequest(req: any): Promise<unknown> {
  switch (req.op) {
    case "list_sets":
      return Promise.resolve(["Friday Gig"]);
    case "list_songs":
      return Promise.resolve([]);
    case "list_pedals":
      return Promise.resolve([]);
    case "get_set":
      return Promise.resolve({ name: req.name, songs: [] });
    default:
      return Promise.resolve(null);
  }
}

beforeEach(() => {
  t.request.mockReset().mockImplementation(routeRequest as any);
});

describe("JSON view", () => {
  it("loads and renders the aggregated config JSON on mount", async () => {
    render(JsonView);
    expect(await screen.findByDisplayValue(/Friday Gig/)).toBeTruthy();
  });

  it("queries all three lists", async () => {
    render(JsonView);
    await screen.findByDisplayValue(/Friday Gig/);
    const ops = t.request.mock.calls.map((c) => (c[0] as any).op);
    expect(ops).toContain("list_sets");
    expect(ops).toContain("list_songs");
    expect(ops).toContain("list_pedals");
  });

  it("Refresh re-queries the device", async () => {
    render(JsonView);
    await screen.findByDisplayValue(/Friday Gig/);
    const before = t.request.mock.calls.length;
    await fireEvent.click(screen.getByRole("button", { name: /refresh/i }));
    await screen.findByDisplayValue(/Friday Gig/);
    expect(t.request.mock.calls.length).toBeGreaterThan(before);
  });

  it("shows an error if loading fails", async () => {
    t.request.mockReset().mockRejectedValue(new Error("link down"));
    render(JsonView);
    expect(await screen.findByText(/link down/)).toBeTruthy();
  });
});
