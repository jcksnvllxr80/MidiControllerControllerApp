# DESIGN.md — MidiController Controller App

The design system of record. All UI decisions calibrate against this. The tokens
live in `src/app.css`; this file is the human-readable contract.

## Aesthetic

**Studio rack / instrument panel.** This is gear, not a web dashboard. Warm
near-black surfaces, one amber "tube-glow" accent, a green "live" indicator, real
typography, and mono LCD-style readouts. Calm by default; the accent is rationed.

Classifier: **App UI** (workspace-driven, task-focused) — not a marketing page.
App UI rules apply: calm surface hierarchy, few colors, minimal chrome, utility
copy. No decorative gradients, no card mosaics, no ornamental icons.

## Type

Two families, both bundled offline via `@fontsource` (no network at runtime):

| Role | Family | Usage |
|---|---|---|
| UI / display | **Space Grotesk** (variable) | headings, body, buttons, brand |
| Mono | **IBM Plex Mono** | LCD readout, device/firmware tags, values, JSON, `.eyebrow` micro-labels |

- Never `system-ui` as the primary face.
- Type scale (~1.25): `--t-2xs 11` · `--t-xs 12` · `--t-sm 13` · `--t-base 15` · `--t-lg 18` · `--t-xl 22` · `--t-2xl 28`.
- Headings: weight 600, `letter-spacing -0.01em`, `text-wrap: balance`.
- `.eyebrow` = mono, 11px, uppercase, `0.14em` tracking — the instrument-panel section labels.

## Color (tokens)

Surfaces layer by elevation, never by lightness flips.

| Token | Hex | Use |
|---|---|---|
| `--bg` | `#0e0e10` | app background |
| `--panel` / `--panel-2` | `#16161a` / `#1d1d22` | panels, hover |
| `--control` | `#232329` | button rest |
| `--inset` | `#08080a` | LCD / code wells |
| `--line` / `--line-strong` | `#2a2a31` / `#3b3b44` | hairlines |
| `--text` | `#ededf1` | primary text |
| `--text-dim` | `#9a9aa6` | secondary (6.9:1 on bg) |
| `--text-faint` | `#85858f` | micro-labels (5.3:1 on bg — WCAG AA) |
| `--accent` / `--accent-press` | `#f5a524` / `#d8890f` | the one accent (amber) |
| `--live` | `#46d39a` | connected indicator |
| `--danger` | `#ff6b6b` | errors |

One accent. Amber is for the primary action and "this is active," nothing else.

### Theming

Because the whole UI reads from these tokens, a theme is just a different set of values applied
via `[data-theme="…"]` — the amber/near-black above is the default ("Studio"), applied as the bare
`:root` by *removing* the attribute. Named themes: **Dracula · Nord · Tokyo Night · Gruvbox** (dark)
and **Solarized Light · Catppuccin Latte** (light), plus **Match-OS** (follows
`prefers-color-scheme`, swapping Studio ⇄ Solarized Light). Picked from the **Appearance** view (a
sidebar item) and persisted to `localStorage`. Each remaps the same tokens (the accent shifts from
amber to the theme's signature color) — components don't change. Tokens live in `src/app.css`; the
store + persistence + OS-follow logic in `src/lib/theme.ts`. The scope is the bare attribute (not
`:root`) so the Appearance preview swatches can self-theme via a nested `data-theme`.

## Spacing & shape

- Spacing scale on 4px base: `--s1 4` … `--s8 48`. No arbitrary pixel values.
- Radius hierarchy (not one bubbly radius): `--r-sm 6` · `--r-md 9` · `--r-lg 14`.

## Components

- **Buttons** — `--control` surface, `--line` border; `.primary` = amber fill with
  dark ink. `focus-visible` shows a 2px amber ring (never `outline:none` alone).
- **Sidebar** (`Sidebar.svelte`) — the primary nav: a collapsible left rail (56px icon-only
  ⇄ 212px icon + label, toggled by a chevron centered on the border, state persisted). Active
  item = `--accent-soft` + amber text + a **left accent bar** (`inset 2px 0 0 var(--accent)`) +
  `aria-current="page"`.
- **Title bar** (`TitleBar.svelte`) — custom frameless bar (`decorations:false`): brand
  wordmark, a live connection pill (green dot + "Connected · <where>" / dim "Disconnected"),
  and minimize / maximize / close controls (close → `--danger` on hover).
- **Tabs** (`.tab`, the Configure Sets/Songs/Pedals switcher) — horizontal, and deliberately
  uses the **same** active treatment as the sidebar item (accent-soft + amber + left accent
  bar) so the two indicators read identically.
- **LED dot** (`.led.live` / `.led.amber`) — `.live` (green) is the connection indicator (the
  title-bar pill + the Control readout); `.amber` is the brand mark on the Connect screen.
- **Notice** (`.notice.err` / `.notice.warn`) — designed error/status banners with
  an icon and `role="alert"`/`role="status"`. Replaces raw red exception text.
- **LCD** — the Control display: `--inset` well, mono, amber text with a soft glow.
- **Device card** — neutral icon tile that warms to amber on hover; name + protocol
  eyebrow + identity. No colored-circle icons.

## Interaction states (every feature specifies all five)

| State | Pattern |
|---|---|
| Loading | explicit label ("Scanning…", "Loading…"); button disabled |
| Empty | warm message + a primary action (e.g. Configure "No sets yet → + New set") |
| Error | `.notice err`, humanized via `lib/errors.ts` (cause + next step), `role="alert"` |
| Success | inline `role="status"` confirmation ("Saved …") |
| Connection lost | heartbeat (5s) detects drop → banner on Connect + return to scan |

Errors are never raw `String(e)`. `lib/errors.ts` strips the `Error:` prefix and
maps known transport failures to plain language with a next step.

## Accessibility

- Contrast: body/secondary text ≥ 4.5:1 (`--text-faint` is the floor at 5.3:1).
- Landmarks: `<nav aria-label="Views">`, `<main>` for view content.
- `aria-current="page"` on the active sidebar item / Configure tab; sidebar buttons keep a
  permanent `aria-label` so they stay reachable when collapsed to icons.
- `focus-visible` ring on every interactive element; touch/click targets ≥ 40px.
- Live regions: status/error use `role="alert"` / `aria-live="polite"`.
- `prefers-reduced-motion`: transitions and hover lifts collapse to instant.

## Motion

Restrained. Hover lifts (`translateY(-1px)`), 0.1–0.15s ease. No drifting
gradients, blobs, or decorative animation. All motion respects reduced-motion.

## Anti-patterns (do not ship)

Blue→purple gradients, `system-ui` as primary font, 3-column icon-in-circle grids,
emoji as controls, decorative card grids, centered-everything, raw exception text
in the UI.
