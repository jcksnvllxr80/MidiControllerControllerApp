<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    scanDevices,
    connectDevice,
    findBootloader,
    flashFirmware,
    type BootloaderDrive,
  } from "../lib/transport";
  import { PROTOCOL_LABEL, type DeviceInfo } from "../lib/protocol";
  import { connectionError } from "../lib/stores";
  import { humanizeError } from "../lib/errors";

  let devices: DeviceInfo[] = [];
  let scanning = false;
  let connectingId: string | null = null;
  let error = "";
  let poll: ReturnType<typeof setInterval> | undefined;

  // RP2350 bootloader (appears after "Update firmware"); poll alongside scans.
  let bootloader: BootloaderDrive | null = null;
  let uf2Path = "";
  let flashing = false;
  let flashMsg = "";
  let flashErr = "";

  // Vite resolves the per-protocol device images at build time.
  const images = import.meta.glob("../assets/devices/*.svg", {
    eager: true,
    query: "?url",
    import: "default",
  }) as Record<string, string>;
  function imageFor(key: string): string {
    return images[`../assets/devices/${key}.svg`] ?? images["../assets/devices/mock.svg"];
  }

  // Signature of the rendered list, so a poll that finds nothing new touches
  // no DOM (no flicker). Includes identity so an identify change still updates.
  let signature = "";
  function sigOf(list: DeviceInfo[]): string {
    return list.map((d) => `${d.id}:${d.identity?.firmware ?? ""}`).join("|");
  }

  // `silent` = background poll: don't flip the button or surface scan errors,
  // and only reassign `devices` when the set actually changed.
  async function scan(silent = false) {
    if (!silent) scanning = true;
    try {
      const found = await scanDevices();
      const nextSig = sigOf(found);
      if (nextSig !== signature) {
        signature = nextSig;
        devices = found;
      }
      error = "";
    } catch (e) {
      if (!silent) error = humanizeError(e);
    } finally {
      if (!silent) scanning = false;
    }
    // Poll for the RP2350 bootloader drive too (present after Update firmware).
    try {
      bootloader = await findBootloader();
    } catch {
      /* command unavailable / no drive — leave as-is */
    }
  }

  async function flash() {
    if (!uf2Path.trim()) {
      flashErr = "Enter the path to your firmware .uf2.";
      return;
    }
    flashing = true;
    flashErr = "";
    flashMsg = "";
    try {
      const dest = await flashFirmware(uf2Path.trim());
      flashMsg = `Flashed ${dest}. The controller will reboot and reconnect.`;
    } catch (e) {
      flashErr = humanizeError(e);
    } finally {
      flashing = false;
    }
  }

  async function connect(d: DeviceInfo) {
    connectingId = d.id;
    error = "";
    connectionError.set(""); // clear any prior lost-connection banner
    try {
      await connectDevice(d);
      // On success the connection store flips and App swaps the view.
    } catch (e) {
      error = humanizeError(e);
    } finally {
      connectingId = null;
    }
  }

  onMount(() => {
    scan();
    poll = setInterval(() => scan(true), 4000); // quietly catch hot-plugs
  });
  onDestroy(() => {
    if (poll) clearInterval(poll);
  });
</script>

<div class="connect">
  <header class="head">
    <div class="brand">
      <span class="led amber"></span>
      <h1>MidiController</h1>
    </div>
    <p class="eyebrow">Choose a controller to connect</p>
  </header>

  {#if $connectionError}
    <div class="notice warn" role="status">
      <span class="ic">⚠</span><span>{$connectionError} Scan to reconnect.</span>
    </div>
  {/if}
  {#if error}
    <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
  {/if}

  {#if bootloader}
    <div class="panel bootloader">
      <div class="bl-head">
        <span class="led amber"></span>
        <strong>RP2350 bootloader — ready to flash</strong>
      </div>
      <p class="muted bl-where">
        Mounted at <span class="mono">{bootloader.mount_point}</span>. Point to your firmware
        <span class="mono">.uf2</span> and flash.
      </p>
      <div class="bl-form">
        <input
          type="text"
          bind:value={uf2Path}
          placeholder={"…\\build-pico\\midicontroller_pico.uf2"}
          aria-label="Firmware .uf2 path"
          spellcheck="false"
        />
        <button class="primary" on:click={flash} disabled={flashing}>
          {flashing ? "Flashing…" : "Flash"}
        </button>
      </div>
      {#if flashErr}
        <div class="notice err" role="alert"><span class="ic">⚠</span><span>{flashErr}</span></div>
      {/if}
      {#if flashMsg}
        <div class="notice warn" role="status"><span class="ic">✓</span><span>{flashMsg}</span></div>
      {/if}
    </div>
  {/if}

  <div class="panel picker">
    <ul class="devices">
      {#each devices as d (d.id)}
        <li class="row">
          <span class="tile">
            <span
              class="device-img"
              role="img"
              aria-label={d.protocol}
              style="mask-image:url({imageFor(d.image)});-webkit-mask-image:url({imageFor(d.image)})"
            ></span>
          </span>
          <span class="meta">
            <span class="name" title={d.name}>{d.name}</span>
            <span class="sub mono">
              <span class="proto">{PROTOCOL_LABEL[d.protocol]}</span>{#if d.identity} · fw {d.identity.firmware}{/if}
            </span>
          </span>
          <button class="connect-btn" on:click={() => connect(d)} disabled={connectingId === d.id}>
            {connectingId === d.id ? "Connecting…" : "Connect"}
          </button>
        </li>
      {/each}

      {#if devices.length === 0}
        <li class="empty">
          <span class="pulse" class:on={scanning}></span>
          <p class="empty-title">{scanning ? "Searching…" : "No devices found"}</p>
          <p class="muted">
            {scanning
              ? "Looking for controllers on serial and USB."
              : "Plug in the controller and scan again."}
          </p>
        </li>
      {/if}
    </ul>

    <div class="toolbar">
      <button class="primary" on:click={() => scan()} disabled={scanning}>
        {scanning ? "Scanning…" : "Scan for devices"}
      </button>
      <span class="status mono">{devices.length} found · Serial · USB · Wi-Fi/Ethernet soon</span>
    </div>
  </div>
</div>

<style>
  .connect {
    max-width: 560px;
    margin: 0 auto;
    padding: var(--s8) var(--s4) var(--s5);
    display: flex;
    flex-direction: column;
    gap: var(--s4);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .brand h1 {
    font-size: var(--t-2xl);
  }
  .head .eyebrow {
    margin: var(--s2) 0 0;
  }

  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    overflow: hidden;
  }

  /* RP2350 bootloader flash card */
  .bootloader {
    border-color: var(--accent-line);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .bl-head {
    display: flex;
    align-items: center;
    gap: var(--s2);
  }
  .bl-where {
    margin: 0;
    font-size: var(--t-sm);
  }
  .bl-form {
    display: flex;
    gap: var(--s2);
  }
  .bl-form input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--t-sm);
  }

  .devices {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--s3);
    padding: var(--s3) var(--s4);
    border-bottom: 1px solid var(--line);
    transition: background 0.14s ease;
  }
  .row:hover {
    background: var(--panel-2);
  }
  .tile {
    flex: none;
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    background: var(--inset);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
  }
  .device-img {
    width: 24px;
    height: 24px;
    background-color: var(--text-dim);
    mask-repeat: no-repeat;
    -webkit-mask-repeat: no-repeat;
    mask-position: center;
    -webkit-mask-position: center;
    mask-size: contain;
    -webkit-mask-size: contain;
    transition: background-color 0.14s ease;
  }
  .row:hover .device-img {
    background-color: var(--accent);
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .name {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: var(--t-2xs);
    color: var(--text-dim);
  }
  .proto {
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .connect-btn {
    flex: none;
    font-size: var(--t-sm);
    color: var(--accent);
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }
  .connect-btn:hover {
    background: var(--accent);
    color: var(--accent-ink);
    border-color: transparent;
  }

  /* Empty / searching */
  .empty {
    list-style: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s2);
    padding: var(--s8) var(--s4);
    text-align: center;
  }
  .empty-title {
    margin: 0;
    font-weight: 600;
  }
  .empty .muted {
    margin: 0;
    font-size: var(--t-sm);
  }
  .pulse {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--line-strong);
    margin-bottom: var(--s2);
  }
  .pulse.on {
    background: var(--accent);
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      box-shadow: 0 0 0 0 var(--accent-soft);
      opacity: 1;
    }
    50% {
      box-shadow: 0 0 0 8px transparent;
      opacity: 0.5;
    }
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--s4);
    padding: var(--s4);
    background: var(--bg);
  }
  .status {
    font-size: var(--t-2xs);
    color: var(--text-dim);
  }
</style>
