<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { scanDevices, connectDevice } from "../lib/transport";
  import { PROTOCOL_LABEL, type DeviceInfo } from "../lib/protocol";

  let devices: DeviceInfo[] = [];
  let scanning = false;
  let connectingId: string | null = null;
  let error = "";
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
      if (!silent) error = String(e);
    } finally {
      if (!silent) scanning = false;
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

  onMount(() => {
    scan();
    poll = setInterval(() => scan(true), 4000); // quietly catch hot-plugs
  });
  onDestroy(() => {
    if (poll) clearInterval(poll);
  });
</script>

<div class="connect">
  <header class="hero">
    <div class="brand">
      <span class="led amber"></span>
      <h1>MidiController</h1>
    </div>
    <p class="eyebrow">Hardware controller · choose a connection to begin</p>
  </header>

  <div class="toolbar">
    <button class="primary" on:click={() => scan()} disabled={scanning}>
      {scanning ? "Scanning…" : "Scan for devices"}
    </button>
    <span class="status mono">{devices.length} found · Serial · USB · Wi-Fi/Ethernet soon</span>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if devices.length === 0 && !scanning}
    <div class="empty">
      <p class="empty-title">No devices found</p>
      <p class="muted">Plug in the controller and scan again.</p>
    </div>
  {/if}

  <ul class="device-grid">
    {#each devices as d (d.id)}
      <li class="device-card">
        <span class="tile">
          <span
            class="device-img"
            role="img"
            aria-label={d.protocol}
            style="mask-image:url({imageFor(d.image)});-webkit-mask-image:url({imageFor(d.image)})"
          ></span>
        </span>
        <div class="device-meta">
          <strong title={d.name}>{d.name}</strong>
          <span class="proto eyebrow">{PROTOCOL_LABEL[d.protocol]}</span>
          {#if d.identity}
            <span class="fw mono">{d.identity.name} · {d.identity.firmware}</span>
          {/if}
        </div>
        <button
          class="connect-btn"
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
    max-width: 760px;
    margin: 0 auto;
    padding: var(--s8) var(--s4) var(--s5);
  }
  .hero {
    padding-bottom: var(--s5);
    border-bottom: 1px solid var(--line);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .brand h1 {
    font-size: var(--t-2xl);
  }
  .hero .eyebrow {
    margin: var(--s3) 0 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--s4);
    margin: var(--s5) 0 var(--s4);
  }
  .status {
    font-size: var(--t-xs);
    color: var(--text-dim);
  }
  .empty {
    padding: var(--s6) 0;
    text-align: center;
  }
  .empty-title {
    margin: 0 0 var(--s1);
    font-weight: 600;
  }
  .empty .muted {
    margin: 0;
    font-size: var(--t-sm);
  }
  .device-grid {
    list-style: none;
    padding: 0;
    margin: var(--s2) 0 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(248px, 1fr));
    gap: var(--s3);
  }
  .device-card {
    display: flex;
    align-items: center;
    gap: var(--s3);
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s3);
    transition:
      border-color 0.15s ease,
      transform 0.1s ease,
      background 0.15s ease;
  }
  .device-card:hover {
    border-color: var(--line-strong);
    background: var(--panel-2);
    transform: translateY(-1px);
  }
  .tile {
    flex: none;
    width: 46px;
    height: 46px;
    display: grid;
    place-items: center;
    background: var(--inset);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
  }
  .device-img {
    width: 26px;
    height: 26px;
    background-color: var(--text-dim);
    mask-repeat: no-repeat;
    -webkit-mask-repeat: no-repeat;
    mask-position: center;
    -webkit-mask-position: center;
    mask-size: contain;
    -webkit-mask-size: contain;
    transition: background-color 0.15s ease;
  }
  .device-card:hover .device-img {
    background-color: var(--accent);
  }
  .device-meta {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
  }
  .device-meta strong {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 600;
  }
  .proto {
    color: var(--text-faint);
  }
  .fw {
    font-size: var(--t-2xs);
    color: var(--text-dim);
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
</style>
