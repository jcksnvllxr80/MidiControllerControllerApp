<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { scanDevices, connectDevice, onDeviceFound } from "../lib/transport";
  import { PROTOCOL_LABEL, type DeviceInfo } from "../lib/protocol";

  let devices: DeviceInfo[] = [];
  let scanning = false;
  let connectingId: string | null = null;
  let error = "";
  let unlisten: UnlistenFn | undefined;
  let poll: ReturnType<typeof setInterval> | undefined;

  // Vite resolves the per-protocol device images at build time.
  const images = import.meta.glob("../assets/devices/*.svg", {
    eager: true,
    query: "?url",
    import: "default",
  }) as Record<string, string>;
  function imageFor(key: string): string {
    return images[`../assets/devices/${key}.svg`] ?? images["../assets/devices/mock.svg"];
  }

  function mergeDevice(d: DeviceInfo) {
    if (!devices.some((x) => x.id === d.id)) {
      devices = [...devices, d];
    }
  }

  async function scan() {
    scanning = true;
    error = "";
    try {
      const found = await scanDevices();
      // Replace the list each scan so unplugged devices drop off.
      devices = found;
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }

  async function connect(d: DeviceInfo) {
    connectingId = d.id;
    error = "";
    try {
      await connectDevice(d);
      // On success the connection store flips and App swaps the view.
    } catch (e) {
      error = String(e);
    } finally {
      connectingId = null;
    }
  }

  onMount(async () => {
    unlisten = await onDeviceFound(mergeDevice);
    await scan();
    poll = setInterval(scan, 4000); // poll so hot-plugged devices appear
  });
  onDestroy(() => {
    if (poll) clearInterval(poll);
    unlisten?.();
  });
</script>

<div class="connect">
  <div class="banner gradient-banner">
    <h1>MidiController</h1>
    <p>Select a connection to your controller</p>
  </div>

  <div class="toolbar">
    <button class="primary" on:click={scan} disabled={scanning}>
      {scanning ? "Scanning…" : "Scan for devices"}
    </button>
    <span class="muted">{devices.length} found · Serial · USB · Wi-Fi/Ethernet (soon)</span>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if devices.length === 0 && !scanning}
    <p class="muted empty">No devices found. Plug in the controller and scan again.</p>
  {/if}

  <ul class="device-grid">
    {#each devices as d (d.id)}
      <li class="device-card">
        <span
          class="device-img"
          role="img"
          aria-label={d.protocol}
          style="mask-image:url({imageFor(d.image)});-webkit-mask-image:url({imageFor(d.image)})"
        ></span>
        <div class="device-meta">
          <strong title={d.name}>{d.name}</strong>
          <span class="proto">{PROTOCOL_LABEL[d.protocol]}</span>
          {#if d.identity}
            <span class="muted small">{d.identity.name} · fw {d.identity.firmware}</span>
          {/if}
        </div>
        <button
          class="primary"
          on:click={() => connect(d)}
          disabled={connectingId === d.id}
        >
          {connectingId === d.id ? "Connecting…" : "Connect"}
        </button>
      </li>
    {/each}
  </ul>
</div>

<style>
  .connect {
    max-width: 820px;
    margin: 0 auto;
    padding: 1.25rem;
  }
  .banner {
    border-radius: var(--radius);
    padding: 1.6rem 1.4rem;
    box-shadow: var(--shadow);
  }
  .banner h1 {
    margin: 0;
    font-size: 1.7rem;
  }
  .banner p {
    margin: 0.25rem 0 0;
    opacity: 0.9;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    margin: 1.2rem 0 0.6rem;
  }
  .empty {
    padding: 1.5rem 0;
  }
  .device-grid {
    list-style: none;
    padding: 0;
    margin: 0.5rem 0 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 0.9rem;
  }
  .device-card {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.9rem;
  }
  .device-img {
    width: 42px;
    height: 42px;
    flex: none;
    background-color: var(--accent);
    mask-repeat: no-repeat;
    -webkit-mask-repeat: no-repeat;
    mask-position: center;
    -webkit-mask-position: center;
    mask-size: contain;
    -webkit-mask-size: contain;
  }
  .device-meta {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    flex: 1;
  }
  .device-meta strong {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .proto {
    font-size: 0.78rem;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .small {
    font-size: 0.76rem;
  }
</style>
