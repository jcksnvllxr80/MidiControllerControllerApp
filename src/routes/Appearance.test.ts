import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

import Appearance from "./Appearance.svelte";
import { theme } from "../lib/theme";

beforeEach(() => {
  localStorage.clear();
  theme.set("studio");
});

describe("Appearance screen", () => {
  it("renders a card for every theme", () => {
    render(Appearance);
    for (const name of ["Studio", "Dracula", "Nord", "Tokyo Night", "Gruvbox", "Solarized Light", "Catppuccin Latte", "Match OS"]) {
      expect(screen.getByRole("button", { name: new RegExp(name) })).toBeTruthy();
    }
  });

  it("selecting a theme sets data-theme on <html>, updates the store, and persists", async () => {
    render(Appearance);
    await fireEvent.click(screen.getByRole("button", { name: /Dracula/ }));
    expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");
    expect(localStorage.getItem("theme")).toBe("dracula");
    expect(screen.getByRole("button", { name: /Dracula/ }).getAttribute("aria-pressed")).toBe("true");
  });

  it("selecting the default Studio removes data-theme (bare :root)", async () => {
    theme.set("nord");
    render(Appearance);
    await fireEvent.click(screen.getByRole("button", { name: /Studio/ }));
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    expect(localStorage.getItem("theme")).toBe("studio");
  });
});
