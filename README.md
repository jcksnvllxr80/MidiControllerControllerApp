# MidiController Controller App

Native desktop app for configuring and controlling the MidiController, built with
**Tauri v2 + Svelte + TypeScript**. It replaces the old web app + HTTP-to-a-Pi bridge:
the controller is now reached over a **pluggable transport** (Serial / USB now;
Wi-Fi / Ethernet later), discovered by scanning on connect.

See [`docs/plan.md`](docs/plan.md) for the full design and roadmap.

## Architecture at a glance

```
Svelte UI ──invoke()/listen()──▶ Rust commands ──▶ Transport trait
(Connect/Control/                 (scan/connect/        ├─ SerialTransport (serialport)
 Configure/JSON)                   disconnect/          ├─ UsbTransport   (nusb)
                                    send_request)        ├─ MockTransport  (in-memory)
                                                         └─ Wi-Fi/Ethernet (future)
                                                              │ newline-JSON wire protocol
                                                              ▼  firmware IConfigTransport
```

The UI talks only to `src/lib/transport.ts`; it never knows which link is in use.
The wire protocol (`src-tauri/src/protocol/`) is one framed JSON request/response channel
that unifies the web app's old config (:8081) and control (:8090) HTTP APIs, and is meant to
match the firmware's `IConfigTransport` (see the `MidiControllerCpp` repo).

## Project layout

```
src/                       Svelte frontend
  main.ts · App.svelte
  routes/                  Connect · Control · Configure · JsonView
  lib/                     transport.ts · protocol.ts · stores.ts
  assets/devices/*.svg     per-protocol device-card icons
src-tauri/                 Rust backend
  src/
    commands.rs            scan_devices / connect_device / disconnect_device /
                           send_request / connection_status  (+ events)
    state.rs               registry + active connection (Arc<Mutex>)
    error.rs               AppError
    transport/             mod.rs (Transport trait) · registry.rs ·
                           serial.rs · usb.rs · mock.rs
    protocol/              mod.rs (Request/Response) · codec.rs (JSON-lines framing)
docs/plan.md               design + roadmap
```

## Prerequisites

- **Node.js** 18+ (tested on 20.x) and **npm**
- **Rust** stable — `rustc` / `cargo`, installed via [rustup](https://rustup.rs)
- **WebView runtime**:
  - Windows — WebView2 (preinstalled on Windows 11)
  - macOS — WKWebView (built in)
  - Linux — `webkit2gtk` + the standard [Tauri Linux build deps](https://v2.tauri.app/start/prerequisites/)

The Tauri **v2** CLI is pinned in `devDependencies`, so there is nothing to install globally —
always invoke it via `npm run tauri …`. (A globally installed `cargo tauri` may be v1; ignore it.)

## Quick start

```bash
npm install            # frontend deps + the local Tauri v2 CLI
npm run tauri dev      # compiles the Rust backend, starts Vite, opens the app (hot reload)
```

On launch you get a **Connect** screen. Click **Scan** to enumerate devices across all
transports; each shows an image, name, and protocol. A **Mock MidiController (dev)** device is
always present, so you can drive the entire UI with no hardware attached. Set the env var
`MIDICTRL_NO_MOCK=1` before launching to hide it.

## Build

```bash
npm run build          # frontend only -> static assets in dist/
npm run tauri build    # full desktop app + native installers (release)
```

`npm run tauri build` outputs the binary and platform installers under
`src-tauri/target/release/` (bundles in `src-tauri/target/release/bundle/`):

| Platform | Artifacts |
|---|---|
| Windows | `.msi` (WiX) and `.exe` (NSIS) |
| macOS | `.app` and `.dmg` |
| Linux | `.deb` and `.AppImage` |

Backend only (no bundling):

```bash
cd src-tauri && cargo build              # debug
cd src-tauri && cargo build --release    # optimized
```

## Test

```bash
npm test                       # frontend unit tests (Vitest)        — 16 tests
npm run check                  # type-check Svelte + TS (svelte-check) — 0 errors
cd src-tauri && cargo test     # backend unit tests                  — 67 tests
```

Run the whole gate in one line (works in bash and PowerShell 7):

```bash
npm test && npm run check && npm run build && cargo test --manifest-path src-tauri/Cargo.toml
```

What's covered (**83 tests total**):

- **Backend (`cargo test`, 67)** — protocol serde + the exact per-`op` wire strings,
  JSON-lines codec framing (encode / line-read / roundtrip with noise + wrong-id skipping),
  every Mock op, registry fan-out + protocol mapping, Serial/USB discovery shape + connect
  guards, `AppState` connect/disconnect/send/status, and error serialization.
- **Frontend (`npm test`, 16)** — protocol constants, the same 18-op wire contract mirrored
  on the TS side, and the `transport.ts` invoke/listen wrappers (command names, args,
  ok/error handling, event payload forwarding) against a mocked Tauri bridge.

The 18-op wire contract is asserted from **both** languages, so any Rust↔TS drift fails a test.

## Troubleshooting

- **`tauri` runs the wrong version** — always use `npm run tauri …`; the v2 CLI is local to this
  project. A global `cargo tauri` may be an older v1 install.
- **Vite / Node engine errors** — Vite 6 needs Node 18+. On Node 20.10 stay on Vite 6 (not 7);
  `npm install` already pins compatible versions.
- **Scan shows no devices** — the Mock device should always appear; if it doesn't, ensure
  `MIDICTRL_NO_MOCK` is unset. Real serial/USB devices only show when plugged in.
- **`npm run tauri build` fails on Linux** — install the Tauri Linux system dependencies
  (`webkit2gtk`, `librsvg`, …) per the prerequisites link above.

## Status

| Area | State |
|---|---|
| Tauri v2 + Svelte + TS shell, Connect/Control/Configure/JSON | ✅ working |
| Transport trait + registry + JSON-lines protocol + codec | ✅ working |
| MockTransport (full protocol, seeded data) | ✅ working |
| SerialTransport — discover / connect+identify / request | ✅ implemented (needs firmware speaking the wire protocol to round-trip) |
| UsbTransport — **discovery** by VID/PID | ✅ working |
| UsbTransport — connect / request | ⛔ stub, pending the firmware's USB descriptor (CDC vs vendor/HID) — see plan Open Q#1 |
| Wi-Fi / Ethernet transports | ⛔ not started (same trait, future) |
| Configure editor | functional per-entity JSON editor (list/get/write/delete); full bespoke pedal/param UI is a later pass |

## Notes / open coordination

- **Wire protocol** (`src-tauri/src/protocol/`) must match the firmware's `IConfigTransport`.
  It is documented as the source of truth to co-design with the firmware effort.
- **Serial vs USB**: if the firmware enumerates as a USB-CDC virtual COM port, use the
  Serial transport (it covers CDC) and `usb.rs` stays a stub. If it exposes a vendor/HID USB
  interface, fill in `usb.rs` connect/request. Set `FIRMWARE_FILTER` in `usb.rs` to the
  firmware's VID/PID to narrow discovery.
