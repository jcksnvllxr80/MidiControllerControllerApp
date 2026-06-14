# MidiController Controller App

Native desktop app for configuring and controlling the MidiController, built with
**Tauri v2 + Svelte 5 + TypeScript**. It replaces the old web app + HTTP-to-a-Pi bridge:
the controller is reached over a **pluggable transport** (Serial, raw USB, or Wi-Fi),
discovered and confirmed at connect time.

See [`docs/plan.md`](docs/plan.md) for the design rationale and roadmap, and
[`DESIGN.md`](DESIGN.md) for the visual system.

## Architecture at a glance

```
Svelte UI ──invoke()/listen()──▶ Rust commands ─────▶ Transport trait (one per link family)
TitleBar · Sidebar               scan_devices          ├─ SerialTransport (serialport — USB-CDC / UART)
Connect · Control · Configure    connect / disconnect  ├─ UsbTransport    (nusb — raw vendor / WinUSB)
JSON · Wi-Fi · Firmware          send_request          ├─ WifiTransport   (mDNS + TCP :8080)
                                 find_bootloader        ├─ MockTransport   (in-memory — dev / test)
                                 flash_firmware         └─ Ethernet        (future, same trait)
                                                            │ newline-JSON wire protocol (codec.rs)
                                                            ▼ firmware EditorProtocol (MidiControllerCpp)
```

The UI talks only to `src/lib/transport.ts`; it never knows which link is in use.
`scan_devices` enumerates every transport, **confirms each candidate with an `identify`
handshake** (so only real MidiControllers appear), and **dedupes by the device's stable
`device_id`** — one physical unit reachable over USB *and* Wi-Fi shows as a single entry.
The wire protocol (`src-tauri/src/protocol/`) is one framed JSON request/response channel
that matches the firmware's `EditorProtocol` (see the `MidiControllerCpp` repo).

## Project layout

```
src/                        Svelte 5 frontend
  main.ts · App.svelte       app shell (connect gate, view routing)
  TitleBar.svelte            custom frameless title bar — drag region, window controls,
                             live connection pill
  Sidebar.svelte             collapsible left nav (56px icon rail ⇄ 212px icons+labels), persisted
  routes/                    Connect · Control · Configure · JsonView · Wifi · Firmware
  lib/
    transport.ts             invoke()/listen() wrappers + firmware-update commands
    protocol.ts              TS mirror of the Rust Request/Response + data model types
    presets.ts               derive a pedal's preset options from its "Set Preset" definition
    stores.ts                connection · connectionError · sidebarCollapsed (persisted)
    dialog.ts                native .uf2 file picker (tauri-plugin-dialog)
    window.ts                title-bar window controls (guarded — no-ops outside Tauri)
    errors.ts                humanize thrown / transport errors for the UI
  app.css                    design tokens + base styles (see DESIGN.md)
  assets/devices/*.svg       per-protocol device-card icons
src-tauri/                   Rust backend
  src/
    commands.rs              scan_devices / connect_device / disconnect_device / send_request /
                             connection_status / find_bootloader / flash_firmware  (+ events)
    state.rs                 AppState: TransportRegistry + active connection (Arc<Mutex>)
    firmware.rs              RP2350 bootloader-drive detection + .uf2 copy (flashing)
    error.rs                 AppError
    transport/               mod.rs (Transport trait) · registry.rs (discover + identify-confirm +
                             dedupe) · serial.rs · usb.rs · wifi.rs · mock.rs
    protocol/                mod.rs (Request / Response) · codec.rs (newline-JSON framing)
    wire_e2e.rs              end-to-end protocol test harness (fake firmware over a byte stream)
docs/plan.md                 design + roadmap
DESIGN.md                    visual system of record (tokens in src/app.css)
TODOS.md                     deferred design/UX polish
```

## Prerequisites

- **Node.js** 18+ (tested on 20.x) and **npm**
- **Rust** stable — `rustc` / `cargo`, via [rustup](https://rustup.rs)
- **WebView runtime**:
  - Windows — WebView2 (preinstalled on Windows 11)
  - macOS — WKWebView (built in)
  - Linux — `webkit2gtk` + the standard [Tauri Linux build deps](https://v2.tauri.app/start/prerequisites/)

The Tauri **v2** CLI is pinned in `devDependencies`, so there's nothing to install globally —
always invoke it via `npm run tauri …`. (A globally installed `cargo tauri` may be v1; ignore it.)

## Quick start

```bash
npm install            # frontend deps + the local Tauri v2 CLI
npm run tauri dev      # compiles the Rust backend, starts Vite, opens the app (hot reload)
```

On launch you get a custom **title bar** (with a live connection pill) over a **Connect**
screen. Click **Scan** to enumerate devices across every transport; each confirmed
MidiController shows an icon, name, protocol, and firmware. Connect, and a collapsible left
**sidebar** (Control · Configure · JSON · Wi-Fi · Firmware, with Disconnect in the footer)
replaces the device list.

**The Mock device** — a fake **Mock MidiController** that lets you drive the whole UI with no
hardware — is **shown in dev** (`npm run tauri dev`) but **hidden in the installed app**, so end
users never see a fake device. Force it either way with environment variables:
`MIDICTRL_MOCK=1` to show it, `MIDICTRL_NO_MOCK=1` to hide it (`NO_MOCK` wins if both are set).

### Connecting over each link

| Link | When | How to connect |
|---|---|---|
| **Serial** | default firmware (USB-CDC, VID `0x2E8A`) | pick the **Serial** entry (a COM/tty port) |
| **USB** (raw) | firmware built `-DMC_ENABLE_USB_EDITOR` (vendor iface, `0xCAFE:0x4001`, WinUSB) | pick the **USB** entry |
| **Wi-Fi** | after setting credentials over USB | found via mDNS (`_midicontroller._tcp`, `midicontroller.local:8080`) |

Set Wi-Fi credentials once from the **Wi-Fi** view (SSID + password → the device joins on
boot and is then discoverable wirelessly).

### Updating firmware

The **Firmware** view shows the version + device id, with **Reboot** and **Update firmware**
(resets the device into its USB bootloader). In bootloader mode the device appears on the
**Connect** screen as an *"RP2350 bootloader — ready to flash"* card; **Browse** for the
`.uf2` and **Flash** (it copies the file onto the bootloader drive). Flashing always needs a
**USB cable**, even when otherwise connected over Wi-Fi.

## Build an installer (ship it to users)

To hand someone a real, double-click installer — no dev server, no command line, no script —
run **one command**:

```bash
npm run tauri build
```

It produces native installers (a normal click-through wizard) under
`src-tauri/target/release/bundle/`:

| Platform | Give users | Path |
|---|---|---|
| **Windows** | the **NSIS `.exe`** *(recommended)* | `…/bundle/nsis/MidiController Controller_<version>_x64-setup.exe` |
| Windows | `.msi` (for managed / IT deploys) | `…/bundle/msi/*.msi` |
| macOS | `.dmg` | `…/bundle/dmg/*.dmg` |
| Linux | `.AppImage` or `.deb` | `…/bundle/{appimage,deb}/` |

On Windows the **NSIS `.exe`** is the friendliest thing to give a non-technical user: it
**installs per-user with no admin/UAC prompt**, runs a simple **Next → Install** wizard, and
adds **Start Menu + desktop shortcuts**. They double-click it, click through, and then launch
the app like any other program — they never see Node, Rust, or a terminal.

> The first `tauri build` downloads the NSIS toolchain automatically. To brand the app +
> installer icon, run `npm run tauri icon path/to/icon.png` once before building.

### Dev / backend-only builds

```bash
npm run build                                                  # frontend assets only -> dist/
cargo build [--release] --manifest-path src-tauri/Cargo.toml   # backend only, no bundle
```

### Automated builds & releases (CI)

Two GitHub Actions workflows live in `.github/workflows/`:

- **`ci.yml`** — runs `svelte-check`, the frontend tests, and `cargo test` on every commit to `master`.
- **`release.yml`** — on a version tag, builds installers for **Windows, macOS (Intel +
  Apple Silicon), and Linux** in parallel and attaches them to a **draft GitHub Release**.

Cut a release without owning a Mac or Linux box:

```bash
npm version patch        # bumps the version and creates a vX.Y.Z tag
git push --follow-tags   # pushing the tag triggers release.yml
```

Then review the draft Release on GitHub and click **Publish**. (You can also trigger it
manually from the Actions tab.) Installers are unsigned for now — `release.yml` has commented
placeholders for the macOS/Windows signing secrets to drop in once you have certificates.

## Test

```bash
npm test                                          # frontend (Vitest + jsdom)        — 74 tests
npm run check                                     # svelte-check (TS + Svelte)        — 0 errors
cargo test --manifest-path src-tauri/Cargo.toml   # backend unit + e2e               — 139 tests
```

Whole gate in one line (bash or PowerShell 7):

```bash
npm test && npm run check && npm run build && cargo test --manifest-path src-tauri/Cargo.toml
```

What's covered (**213 tests total**):

- **Backend (`cargo test`, 139)** — `Request`/`Response` serde + the exact per-`op` wire
  strings (the **23-op contract**); the newline-JSON codec (encode / line-read / roundtrip;
  noise, wrong-id, stale-id, large/nested payloads); every Mock op with parametric coverage;
  registry fan-out + protocol mapping + **dedupe-by-`device_id`**; Serial VID filter, USB/Wi-Fi
  discovery shape + connect guards; `firmware.rs` flash validation; `AppState`
  connect/disconnect/send/status; error serialization; and an **end-to-end wire harness**
  (`wire_e2e.rs`) running full sessions against a fake firmware over a real byte stream.
- **Frontend (`npm test`, 74)** — protocol constants + the same 23-op contract mirrored in TS;
  `presets.ts` derivation (banks, string options, current-value preservation); the
  `transport.ts` wrappers against a mocked Tauri bridge; and **component tests**
  (Testing Library + jsdom) driving the real Svelte views — Connect (scan → cards → connect,
  empty/error, bootloader flash card), Control, Configure (tabs / list / select / new / save /
  delete + Advanced raw JSON), JSON, Wi-Fi (status / set / enable), Firmware (reboot /
  update), and the App shell (connect gating, sidebar nav, disconnect, live status).

The 23-op wire contract is asserted from **both** languages, so any Rust↔TS drift fails a test.

## Troubleshooting

- **`tauri` runs the wrong version** — always use `npm run tauri …`; the v2 CLI is local.
- **Scan shows no devices** — in a **dev** build the Mock device appears unless
  `MIDICTRL_NO_MOCK=1`; an **installed** build hides it unless `MIDICTRL_MOCK=1`. Real devices
  show only when plugged in. Discovery filters serial ports by **VID** (`0x2E8A` /
  `0xCAFE:0x4001`), not the product string, so a renamed COM port still appears.
- **Device vanished after entering bootloader** — expected: in BOOTSEL mode the Pico is a
  mass-storage drive, not a serial device, so it leaves the device list and shows as the
  bootloader flash card instead.
- **`npm run tauri build` fails on Linux** — install the Tauri Linux system deps
  (`webkit2gtk`, `librsvg`, …) per the prerequisites link.

## Status

| Area | State |
|---|---|
| Tauri v2 + Svelte 5 + TS shell; custom title bar; collapsible sidebar | ✅ working |
| Transport trait + registry (identify-confirm + dedupe-by-`device_id`) + JSON-lines codec | ✅ working |
| MockTransport (full protocol, seeded data; dev-only — hidden in release) | ✅ working |
| SerialTransport — discover (VID filter) / connect+identify / request | ✅ working (confirmed on hardware) |
| UsbTransport — raw vendor/WinUSB discover / connect / request | ✅ implemented (pairs with the `-DMC_ENABLE_USB_EDITOR` firmware build; unverified on hardware) |
| WifiTransport — mDNS discovery + TCP `:8080` | ✅ implemented (app-side verified; not network-tested) |
| Configure editor — Sets/Songs/Pedals GUI (dropdowns, checkboxes, preset banks) + Advanced raw JSON | ✅ working |
| Control · JSON view | ✅ working |
| Wi-Fi setup (`wifi_set` / `wifi_status` / `wifi_enable`) | ✅ working |
| Firmware update + reboot (`reboot` / `reboot_bootloader`, bootloader-drive flash) | ✅ implemented (flash unverified on hardware) |
| **Themes / Appearance** (Dracula, Nord, Tokyo Night, Gruvbox, Solarized Light, Catppuccin Latte, match-OS) | 🚧 planned — next |
| Ethernet transport | ⛔ not started (same trait, future) |

## Notes / coordination

- **Wire protocol** (`src-tauri/src/protocol/`) must match the firmware's `EditorProtocol`;
  the firmware repo's `docs/wifi-app-handoff.md` is the agreed contract. Adding an `op`
  means updating *both* the Rust enum and `src/lib/protocol.ts` (a test enforces parity).
- **Serial vs USB** — the default firmware enumerates as USB-CDC, so use the **Serial**
  transport. The **raw USB** transport pairs with the firmware's vendor build
  (`-DMC_ENABLE_USB_EDITOR`, WinUSB); `FIRMWARE_FILTER` in `usb.rs` narrows discovery to its
  VID/PID.
- **`device_id`** is the RP2350 per-chip unique id surfaced in `identify` — used only to
  dedupe one physical unit seen on multiple transports. It's optional and never an allow-list.
