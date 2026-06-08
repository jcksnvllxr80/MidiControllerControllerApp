# TODOS

Design/UX debt surfaced by `/plan-design-review` and deferred (the higher-impact
items were fixed in-line; these are polish). See `DESIGN.md` for the system.

## Design polish

- [ ] **Skeleton loaders** — Configure/JSON show a text "Loading…"; replace with
      skeleton shapes that match the real list/editor layout so loads don't jump.
      Why: less perceived latency, no layout shift. (P3)
- [ ] **Focus management on view switch** — when the segmented nav changes view,
      move focus to the new `<main>` heading so keyboard/screen-reader users land
      in the right place. Why: keyboard wayfinding. (P2)
- [ ] **Min-width audit (720px)** — the app window min is 720×560; verify the
      Configure `232px + 1fr` panes and the device grid hold without overflow at
      that width. Why: the only "responsive" target a fixed desktop window has. (P3)
- [ ] **Long-name truncation in lists** — Configure list `.name` buttons should
      `text-overflow: ellipsis` for 40+ char set/song names (device cards already
      do). Why: long names currently wrap and break row rhythm. (P3)
- [ ] **Explicit reconnect affordance** — after a lost-connection banner, the user
      relies on the auto-scan. Consider a "Reconnect to <last device>" button that
      remembers the last `DeviceInfo`. Why: one-click recovery. (P2)
- [ ] **Tap/short repeat affordance** — Control sends one op per click; consider a
      pressed/active visual that lingers briefly so rapid presses read as registered.
      Why: footswitch feedback. (P3)

## Notes

- Mobile/touch is explicitly **out of scope** — this is a fixed desktop window
  (Tauri, min 720×560). Don't add hamburger nav or mobile breakpoints.
- USB connect/request remain stubbed pending the firmware USB descriptor (see
  `docs/plan.md` Open Question #1) — not design debt.
