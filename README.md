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

## Develop / run

Prereqs: Node 18+, Rust (stable), and on Windows the WebView2 runtime (preinstalled on Win11).
The Tauri v2 CLI is pinned locally (`devDependencies`), so no global install is needed.

```bash
npm install
npm run tauri dev      # launches the desktop app with hot reload
```

Other commands:

```bash
npm run build          # build the frontend (Vite) into dist/
npm run check          # type-check the Svelte/TS
npm run tauri build    # produce installers (MSI/NSIS on Windows)
cd src-tauri && cargo test   # Rust unit tests (protocol/mock)
```

On launch you get a **Connect** screen. Click **Scan** to enumerate devices across all
transports; each shows an image, name, and protocol. A **Mock MidiController (dev)** device
is always present so you can drive the whole UI with no hardware. (Set `MIDICTRL_NO_MOCK=1`
to hide it.)

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
