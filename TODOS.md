# TODOS

Design/UX debt surfaced by `/plan-design-review` and deferred (the higher-impact
items were fixed in-line; these are polish). See `DESIGN.md` for the system.

## Design polish

- [ ] **Skeleton loaders** — Configure/JSON show a text "Loading…"; replace with
      skeleton shapes that match the real list/editor layout so loads don't jump.
      Why: less perceived latency, no layout shift. (P3)
- [ ] **Focus management on view switch** — when the sidebar (or Configure tabs)
      changes view, move focus to the new `<main>` heading so keyboard/screen-reader
      users land in the right place. Why: keyboard wayfinding. (P2)
- [ ] **Min-width audit (720px)** — the app window min is 720×560; with the sidebar
      (212/56px) verify the Configure `268px + 1fr` panes and the device grid hold
      without overflow at that width. Why: the only "responsive" target a fixed
      desktop window has. (P3)
- [ ] **Per-parameter pedal editing** — the pedal editor shows a preset dropdown
      (sourced from the pedal's "Set Preset" definition) but params still list
      generically; add per-parameter `engaged` checkboxes + value controls sourced
      from each pedal's `Parameters` (e.g. SuperSwitcher loops). Why: full GUI parity
      with the old web app. (P2)
- [ ] **Explicit reconnect affordance** — after a lost-connection banner, the user
      relies on the auto-scan. Consider a "Reconnect to <last device>" button that
      remembers the last `DeviceInfo`. Why: one-click recovery. (P2)
- [ ] **Tap/short repeat affordance** — Control sends one op per click; consider a
      pressed/active visual that lingers briefly so rapid presses read as registered.
      Why: footswitch feedback. (P3)

## Notes

- Mobile/touch is explicitly **out of scope** — this is a fixed desktop window
  (Tauri, min 720×560). Don't add hamburger nav or mobile breakpoints.
- **Themes / Appearance — shipped.** Theme picker (Studio default + Dracula, Nord,
  Tokyo Night, Gruvbox, Solarized Light, Catppuccin Latte, Match-OS) as a 6th sidebar
  item (`src/routes/Appearance.svelte`), remapping the `src/app.css` tokens, persisted
  via `src/lib/theme.ts`. Match-OS follows `prefers-color-scheme`.
- USB connect/request are **implemented** (raw vendor/WinUSB, pairs with the firmware's
  `-DMC_ENABLE_USB_EDITOR` build) — the earlier "stub pending USB descriptor" note is resolved.
