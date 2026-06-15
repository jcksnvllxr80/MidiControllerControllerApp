import { writable } from "svelte/store";

/**
 * Theme system. Every theme is just a different set of values for the CSS
 * design tokens in `app.css` — a theme block (`[data-theme="…"]`) remaps the
 * same `--bg`/`--accent`/… custom properties, so components never change.
 * The default "studio" amber/near-black is the bare `:root`, applied by
 * *removing* the attribute. "match-os" follows `prefers-color-scheme`.
 */
export type ThemeId =
  | "studio"
  | "dracula"
  | "nord"
  | "tokyo-night"
  | "gruvbox"
  | "solarized-light"
  | "catppuccin-latte"
  | "match-os";

export interface ThemeMeta {
  id: ThemeId;
  label: string;
  group: "dark" | "light" | "system";
}

/** Display order in the Appearance view. */
export const THEMES: ThemeMeta[] = [
  { id: "studio", label: "Studio", group: "dark" },
  { id: "dracula", label: "Dracula", group: "dark" },
  { id: "nord", label: "Nord", group: "dark" },
  { id: "tokyo-night", label: "Tokyo Night", group: "dark" },
  { id: "gruvbox", label: "Gruvbox", group: "dark" },
  { id: "solarized-light", label: "Solarized Light", group: "light" },
  { id: "catppuccin-latte", label: "Catppuccin Latte", group: "light" },
  { id: "match-os", label: "Match OS", group: "system" },
];

const KEY = "theme";
const VALID = new Set<string>(THEMES.map((t) => t.id));

function load(): ThemeId {
  try {
    const v = localStorage.getItem(KEY);
    if (v && VALID.has(v)) return v as ThemeId;
  } catch {
    /* no localStorage (test/SSR) */
  }
  return "studio";
}

/** Currently selected theme (the user's literal choice, incl. "match-os"). */
export const theme = writable<ThemeId>(load());

function prefersDark(): boolean {
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch {
    return true;
  }
}

/** Resolve the user's choice to the concrete theme to paint. */
function resolve(id: ThemeId): ThemeId {
  if (id !== "match-os") return id;
  return prefersDark() ? "studio" : "solarized-light";
}

function paint(id: ThemeId) {
  if (typeof document === "undefined") return;
  const resolved = resolve(id);
  const root = document.documentElement;
  if (resolved === "studio") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", resolved);
}

// Track the current choice so the OS-change listener knows whether to repaint.
let current: ThemeId = load();
let mql: MediaQueryList | undefined;
const onOsChange = () => {
  if (current === "match-os") paint("match-os");
};

theme.subscribe((id) => {
  current = id;
  try {
    localStorage.setItem(KEY, id);
  } catch {
    /* in-memory only */
  }
  // Only listen to the OS while actually following it.
  if (typeof window !== "undefined" && window.matchMedia) {
    if (!mql) mql = window.matchMedia("(prefers-color-scheme: dark)");
    mql.removeEventListener?.("change", onOsChange);
    if (id === "match-os") mql.addEventListener?.("change", onOsChange);
  }
  paint(id);
});
