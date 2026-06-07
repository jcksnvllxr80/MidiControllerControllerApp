# MidiController Controller App: Web App → Native Desktop App

> **Status:** Phases 0–2 implemented and building (Tauri v2 + Svelte + TS shell,
> transport interface + JSON-lines protocol, Mock + Serial transports, USB
> discovery). See [`../README.md`](../README.md) for the per-area status table.
> Run with `npm install && npm run tauri dev`.

## Context
The MidiController system has three parts:

1. **Firmware** (`MidiControllerCpp/`) — the pure‑C++ "brain" being ported from Python in a
   parallel effort, headed for a microcontroller (PIC32MZ‑class). It exposes a hardware
   abstraction layer (ports & adapters); config/control will reach it over an
   `IConfigTransport` port (USB, replacing the old WiFi/HTTP bridge). See
   `MidiControllerCpp/docs/plan.md`.
2. **Web app** (`controllerWebApp/`) — the existing vanilla‑JS single‑page UI that edits and
   controls the pedalboard. Today it talks to two HTTP services on the Raspberry Pi:
   - **Config API** `http://<host>:8081` — CRUD for sets / songs / parts / pedals (JSON files).
   - **Control API** `http://<host>:8090/midi_controller` — live `dpad` / `short` button commands.
3. **Controller app** (`MidiControllerControllerApp/` ← *this repo*) — the deliverable: a
   native desktop app that replaces the web app + HTTP bridge, talking to the firmware
   directly over a swappable link (serial / USB now; WiFi / Ethernet later).

**This plan covers #3 only.** The data model (Setlist → Song → Part → Pedal) and the
existing UI carry over unchanged; what changes is the *transport*: HTTP-to-a-Pi becomes a
pluggable on-device link, selected and discovered at connect time.

## Goals (from the request)
- Native, **desktop‑launchable** app (icon, installer) — not a browser tab.
- Connect to the controller over **USB or serial** now.
- **Transport is an interface**, so the link can be swapped or *probed* to see what's
  available: `WiFi`, `USB`, `Ethernet`, `Serial`. Implement **Serial** and **USB** as the
  concrete adapters now; leave WiFi/Ethernet as same‑interface stubs.
- **Discovery by polling on connect**: when the user scans, each transport enumerates
  candidates; a found device shows up as a card with an **image**, its **name**, and its
  **protocol**. Pick one → connect.

---

## Recommendation: **Tauri** (not Electron)

| | **Tauri** ✅ | Electron |
|---|---|---|
| Backend language | **Rust** | Node.js |
| Serial / USB | Rust `serialport` / `nusb` in the backend | `node-serialport` / WebSerial |
| Installer size | **~3–10 MB** | ~120 MB+ (bundles Chromium) |
| Memory | System WebView (WebView2 on Win) | Bundled Chromium per app |
| Security model | Locked‑down, allowlisted commands | Full Node in renderer unless hardened |
| **Proven here?** | **Yes — you already ship `serial-flash-gui`** | No |

**Decisive factor:** you already have a working Tauri + `serialport` desktop app
(`git/serial-flash-gui`, Tauri 1.6, `serialport = "4.3"`, `tokio`, `thiserror`, `windows`)
that flashes your Microchip MCU over serial. This app is the *same stack* aimed at the same
class of device. We reuse a battle‑tested pattern instead of standing up a second runtime.

> **Version (decided):** **Tauri v2** — current stable, capability‑based permissions. The
> Rust `serialport`/`nusb` code ports almost verbatim from `serial-flash-gui` (v1); only
> the `tauri.conf.json` / capability wiring differs between versions.

The frontend is a **ground‑up rewrite in Svelte + Vite + TypeScript** (decided), replacing
the vanilla‑JS/jQuery app. The existing `controllerWebApp` is kept as the **design and
behavior reference**: its three states (Control / Configure / JSON), the Setlist → Song →
Part → Pedal editors, the gradient backdrop, and `css/style.css` are reproduced as Svelte
components, and its `data/*.json` shapes define the typed models. Svelte stores replace the
old committed‑vs‑WIP dictionaries (`$config` / `$wipConfig`), and `@tauri-apps/api`
`invoke()`/`listen()` replaces `$.getJSON`/`$.post`. TypeScript types are shared with the
Rust `ProtocolMessage` surface so the wire contract is checked at compile time.

---

## Architecture

```
┌───────────────────────────── Tauri App ──────────────────────────────┐
│  Frontend (Svelte + Vite + TS)      Backend (Rust)                     │
│  ───────────────────                ───────────────────               │
│  routes/components · stores                                           │
│  ($config $wipConfig $connection)   commands.rs   #[tauri::command]   │
│  lib/transport.ts ─ invoke() ─────▶   scan_devices / connect /        │
│  Connect.svelte    listen()  ◀─events disconnect / send_request       │
│       │                                   │                            │
│       │                              state.rs  (registry + active conn)│
│       ▼                                   │                            │
│  Connect UI: scan → cards            ┌────▼──────────── Transport ─────┐
│  (image · name · protocol) → connect │  trait Transport (the interface)│
└──────────────────────────────────────│  Protocol{ Serial Usb Wifi Eth}│
                                        │  discover / probe / connect /   │
                                        │  request / disconnect           │
                                        └───┬───────┬───────┬─────────────┘
                                  ┌─────────▼─┐ ┌───▼────┐ ┌▼──────────────┐
                                  │Serial     │ │Usb     │ │Mock (dev/test)│
                                  │serialport │ │nusb    │ │  + Wifi/Eth   │
                                  │COM/tty    │ │VID:PID │ │  stubs (later)│
                                  └─────┬─────┘ └───┬────┘ └───────────────┘
                                        └─────┬─────┘
                                              ▼  wire protocol (JSON frames)
                                   ┌──────────────────────┐
                                   │ Firmware IConfigTrans │  (MidiControllerCpp)
                                   └──────────────────────┘
```

The trait is the contract the request asked for. Each concrete transport answers the same
questions — *what devices are out there, is this one ours, open it, talk to it* — so the UI
and the rest of the app never know or care which link is in use.

### The transport interface (Rust sketch)

```rust
pub enum Protocol { Serial, Usb, Wifi, Ethernet }

/// What a scan turns up before we've confirmed it's ours.
pub struct DeviceInfo {
    pub id: String,          // stable handle, e.g. "serial:COM4" / "usb:1209:0001"
    pub protocol: Protocol,
    pub name: String,        // human label (port name, USB product string, mDNS name)
    pub address: Address,    // Port(String) | Usb{vid,pid,serial} | Net{ip,port}
    pub image: String,       // asset key for the card icon (per device/protocol)
}

/// What the device reports back after a successful identify handshake.
pub struct DeviceIdentity {
    pub name: String,        // device-reported friendly name
    pub firmware: String,    // version string
    pub protocol_version: u16,
}

#[async_trait]
pub trait Transport: Send + Sync {
    fn protocol(&self) -> Protocol;
    async fn discover(&self) -> Vec<DeviceInfo>;               // enumerate candidates
    async fn probe(&self, d: &DeviceInfo) -> Option<DeviceIdentity>; // identify handshake
    async fn connect(&mut self, d: &DeviceInfo) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn request(&mut self, msg: ProtocolMessage) -> Result<ProtocolMessage>;
    fn is_connected(&self) -> bool;
}
```

A `TransportRegistry` holds every concrete transport and runs discovery across all of them:

```rust
impl TransportRegistry {
    async fn discover_all(&self) -> Vec<DeviceInfo>  // fan out over Serial, Usb, (Wifi, Eth)
}
```

**Serial vs USB — why they're genuinely two adapters:**
- `SerialTransport` (crate `serialport`): enumerates **COM/tty ports**, opens by *port name +
  baud*. Covers a USB‑CDC virtual COM port *and* a real UART via an FTDI/CP2102 adapter.
  This is exactly what `serial-flash-gui` already does.
- `UsbTransport` (crate `nusb`, pure‑Rust libusb alt): enumerates by **VID/PID**, talks raw
  bulk/interrupt endpoints (or HID) with no COM port involved — for a firmware that exposes
  a vendor‑specific USB interface instead of CDC.

> If the firmware enumerates **only** as a USB‑CDC virtual COM port, then "USB" and "Serial"
> collapse onto `SerialTransport` and `UsbTransport` becomes optional. This depends on the
> firmware's USB descriptor — flagged in Open Questions and to be settled with the firmware
> agent.

### The wire protocol

Today's two HTTP services collapse into **one framed request/response channel** over the
link. Proposed framing: **newline‑delimited JSON** (one JSON object per line — human‑
debuggable over a serial monitor), each request carrying a correlation `id`:

```jsonc
// request
{"id": 7, "op": "get_set", "name": "Friday Gig"}
// response
{"id": 7, "ok": true, "data": { ...set json... }}
```

`op` covers the full surface the web app uses today (one channel, config + control):

| Existing HTTP | New `op` |
|---|---|
| `GET /sets` · `/set/{n}` | `list_sets` · `get_set` |
| `GET /songs` · `/song/{n}` | `list_songs` · `get_song` |
| `GET /pedals` · `/pedal/{n}` | `list_pedals` · `get_pedal` |
| `POST /{type}/{name}` | `write_set` · `write_song` · `write_part` · `write_pedal` |
| `POST /{type}/delete/{name}` | `delete_{type}` |
| `GET /dpad/{dir}` · `/short/{btn}` | `dpad` · `short` |
| — (new) | `identify` → `DeviceIdentity` (used by `probe`) |

The framing/codec is a small `protocol/codec.rs` so it can be swapped (length‑prefixed,
COBS, CBOR) without touching transports or the UI. **This wire spec must match the
firmware's `IConfigTransport`** — `docs/protocol.md` is the shared source of truth and is
co‑designed with the firmware agent.

### Frontend (Svelte rewrite)

- `lib/transport.ts` — typed wrapper over `@tauri-apps/api` `invoke()` + `listen()`.
  Exposes `request(op, args)` returning typed `ProtocolMessage` responses; TS types mirror
  the Rust enums (kept in `lib/protocol.ts`) so the wire contract is compile‑checked.
- `lib/stores.ts` — Svelte stores replacing the old committed‑vs‑WIP dictionaries:
  `connection` (status + active device), `config` (committed sets/songs/parts/pedals),
  `wipConfig` (in‑progress edits; a "save" promotes WIP → committed via a `write_*` op).
- `routes/Connect.svelte` — the **Connect screen**: "Scan" → renders discovered devices as
  **cards (image · name · protocol)** → "Connect" → live status. Driven by `device-found` /
  `connection-status` events. Gates the rest of the app until connected.
- `routes/Control.svelte`, `Configure.svelte`, `JsonView.svelte` — the three existing
  states, rebuilt as components: dpad + button grid, the Setlist/Song/Part/Pedal editors,
  and the raw‑JSON viewer. `css/style.css` and the Granim gradient carry over as the look.

### Discovery & connection UX (the polling flow)

1. App opens on a **Connect** screen (gates the rest of the UI until connected).
2. **Scan** → backend `scan_devices` runs `registry.discover_all()`; as each transport
   finds candidates it `probe`s them (identify handshake) and **emits a `device-found`
   event** per confirmed device. Frontend appends a card live (image + name + protocol) —
   *polling* re‑runs the scan on an interval while the screen is open so hot‑plugged
   devices appear.
3. **Connect** on a card → `connect(device_id)`; on success the UI transitions to Control.
4. While connected, a lightweight heartbeat (periodic `identify`/ping) emits
   `connection-status` events; a drop surfaces a banner + auto return to Connect.

---

## Project layout (`MidiControllerControllerApp/`)

```
package.json · vite.config.ts · tsconfig.json   # Svelte + Vite + TS frontend
index.html                   # Vite entry
src/                         # Svelte app (rebuilt from controllerWebApp's UI)
  main.ts · App.svelte
  routes/
    Connect.svelte           # scan / device cards / connect / status
    Control.svelte           # dpad + button grid (live control)
    Configure.svelte         # Setlist/Song/Part/Pedal editors
    JsonView.svelte          # raw JSON viewer
  lib/
    transport.ts             # invoke()/listen() wrapper, typed request(op,args)
    protocol.ts              # TS mirror of Rust ProtocolMessage + data models
    stores.ts                # connection / config / wipConfig stores
  styles/style.css           # carried over from controllerWebApp
  assets/                    # icons + per-protocol/device card images
src-tauri/
  Cargo.toml                 # tauri v2, serialport, nusb, tokio, serde, thiserror, anyhow
  tauri.conf.json
  build.rs
  icons/
  src/
    main.rs                  # builder, managed state, command + event registration
    state.rs                 # AppState: TransportRegistry + active connection (RwLock)
    commands.rs              # #[command] scan_devices / connect / disconnect / send_request
    transport/
      mod.rs                 # Transport trait, Protocol, DeviceInfo, DeviceIdentity, Address
      registry.rs            # discover_all() fan-out
      serial.rs              # SerialTransport (serialport)
      usb.rs                 # UsbTransport (nusb)
      mock.rs                # MockTransport — in-memory device for dev & tests
      wifi.rs · ethernet.rs  # later — same trait, stubbed
    protocol/
      mod.rs                 # ProtocolMessage request/response enums
      codec.rs               # framing (JSON-lines) + id correlation
docs/
  plan.md                    # this file
  protocol.md                # wire spec — shared with firmware agent
```

---

## Phases

**Phase 0 — Scaffold & rebuild the UI (shippable shell).**
Init Tauri v2 + Svelte/Vite/TS in this repo (`create-tauri-app`, Svelte template). Rebuild
the `controllerWebApp` UI as Svelte components (Connect/Control/Configure/JsonView), porting
`style.css` and the gradient backdrop. Add a `MockTransport` (returns the existing
`data/*.json` shapes and canned `dpad`/`short` display text) plus `transport.ts` so the
*entire UI runs end‑to‑end against the mock* — proves parity before any hardware.
**Done when:** app launches as a native window, scan shows a fake device card, connect →
full Control/Configure/JSON flow works.

**Phase 1 — Transport interface + protocol + commands.**
Define the `Transport` trait, `Protocol`/`DeviceInfo`/`DeviceIdentity`, `TransportRegistry`,
and `ProtocolMessage` + `codec.rs`. Wire the four Tauri commands and the `device-found` /
`connection-status` events. Mock implements the full trait. **Done when:** the registry
fan‑out + event‑driven Connect UI work against the mock with the real command surface.

**Phase 2 — SerialTransport (first real link).**
Implement discovery (`serialport::available_ports`), `probe` (open + `identify`), `connect`,
and framed `request` over the port. Reuse `serial-flash-gui`'s port handling. **Done when:**
the app scans real COM/tty ports, identifies a firmware on a loopback/dev board, and drives
config + control over serial.

**Phase 3 — UsbTransport (raw USB).**
Implement `nusb` enumeration by VID/PID, endpoint open, and framed `request`. Same trait,
same UI. **Done when:** a vendor‑USB firmware is discovered and driven without a COM port.
*(May reduce to a thin CDC note if the firmware is CDC‑only — see Open Questions.)*

**Phase 4 — Hardening & packaging.**
Reconnect/heartbeat polish, error surfacing, settings (remember last device, baud), app
icon, and **installers** (MSI/NSIS on Windows; `.deb`/AppImage on Linux) via `tauri build`.
Optional CI to build artifacts on tag.

**Future — WiFi & Ethernet.**
`wifi.rs` / `ethernet.rs` implement the same trait: discovery via mDNS/zeroconf, transport
over TCP/WebSocket. No UI or core changes — that's the payoff of the interface.

---

## Testing
- **Rust unit:** `codec` round‑trip (encode → decode parity); `ProtocolMessage` ↔ JSON;
  `MockTransport` drives the full `op` surface; registry fan‑out dedups/merges results.
- **Serial integration:** loopback using a virtual port pair (`com0com` on Windows /
  `socat` PTY on Linux) running a tiny firmware stub that speaks the wire protocol; assert a
  full request/response and an `identify` handshake.
- **Frontend:** manual QA via the mock transport (no hardware needed); the existing
  Control/Configure/JSON flows are the acceptance checklist.

## Coordination with the firmware effort
- `docs/protocol.md` is the **shared contract**; the firmware's `IConfigTransport` and this
  app's `protocol/` must implement the same frames + `identify` handshake.
- The firmware's **USB descriptor choice (CDC vs vendor)** decides whether `UsbTransport` is
  a distinct adapter or folds into `SerialTransport`.
- The firmware already standardizes on **JSON** config (per its plan), so the wire payloads
  reuse the same set/song/part/pedal JSON shapes the web app already knows.

## Risks / explicit decisions
- **CDC vs raw USB ambiguity** — designing for both via the trait avoids a rewrite; we drop
  `usb.rs` to a stub if the firmware is CDC‑only.
- **One channel, two old APIs** — config (8081) + control (8090) unify into one framed link;
  `op` namespacing keeps them distinct without two sockets.
- **Svelte rewrite over vanilla port** — chosen for a clean, typed, long‑term base. Risk is
  UI regression vs the existing app; mitigated by keeping `controllerWebApp` as the live
  reference and matching it state‑by‑state against the mock transport before hardware.

## Decisions (locked)
- **Framework:** Tauri **v2**.
- **Frontend:** ground‑up **Svelte + Vite + TypeScript** rewrite; `controllerWebApp` is the
  design/behavior reference.
- **Transports:** build **both** `SerialTransport` and `UsbTransport` behind the trait; if
  the firmware turns out CDC‑only, `usb.rs` collapses to a stub.

## Open questions (see chat)
1. **USB descriptor:** once the firmware's USB is implemented, confirm CDC‑virtual‑COM vs
   vendor/HID so we keep or stub `UsbTransport`. (Designing for both until then.)
2. **Wire‑protocol ownership:** this app proposes `docs/protocol.md` now, wait for the
   firmware's `IConfigTransport` spec, or co‑design?
3. **Source UI confirmation:** `controllerWebApp` is assumed as the reference UI — confirm
   it's the right one (vs the older `MidiControllerWebApp`).
```