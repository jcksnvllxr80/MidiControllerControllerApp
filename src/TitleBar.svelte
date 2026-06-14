<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { connection } from "./lib/stores";
  import { minimize, toggleMaximize, closeWindow, isMaximized, onResized } from "./lib/window";
  import type { DeviceInfo } from "./lib/protocol";

  let maximized = false;
  let unlisten: (() => void) | undefined;

  onMount(async () => {
    maximized = await isMaximized();
    unlisten = await onResized(async () => {
      maximized = await isMaximized();
    });
  });
  onDestroy(() => unlisten?.());

  $: connected = $connection.connected;

  // A short "where" for the status pill: COM4 / midicontroller.local / USB.
  function locator(d?: DeviceInfo): string {
    if (!d) return "";
    const a = d.address;
    if (a.kind === "port") return a.name;
    if (a.kind === "net") return a.host;
    if (a.kind === "usb") return "USB";
    if (a.kind === "mock") return "mock";
    return "";
  }
  $: where = locator($connection.device);
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="left">
    <span class="wordmark">MidiController</span>
    <span class="pill" class:on={connected} title={connected ? "Connected" : "Disconnected"}>
      <span class="dot" class:live={connected}></span>
      {#if connected}Connected{#if where} · <span class="mono">{where}</span>{/if}{:else}Disconnected{/if}
    </span>
  </div>

  <div class="controls">
    <button class="ctl" title="Minimize" aria-label="Minimize" on:click={minimize}>
      <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true"><line x1="2" y1="6.5" x2="10" y2="6.5" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button class="ctl" title={maximized ? "Restore" : "Maximize"} aria-label={maximized ? "Restore" : "Maximize"} on:click={toggleMaximize}>
      {#if maximized}
        <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.1">
          <rect x="2.2" y="3.4" width="6" height="6" rx="0.8" /><path d="M4 3.4V2.4a.8.8 0 0 1 .8-.8h4.4a.8.8 0 0 1 .8.8v4.4a.8.8 0 0 1-.8.8h-1" />
        </svg>
      {:else}
        <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.1"><rect x="2.4" y="2.4" width="7.2" height="7.2" rx="0.8" /></svg>
      {/if}
    </button>
    <button class="ctl close" title="Close" aria-label="Close" on:click={closeWindow}>
      <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true" stroke="currentColor" stroke-width="1.2"><line x1="3" y1="3" x2="9" y2="9" /><line x1="9" y1="3" x2="3" y2="9" /></svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 38px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
    padding-left: var(--s4);
    user-select: none;
  }
  /* The left group is visual only — let clicks fall through to the drag region. */
  .left {
    display: flex;
    align-items: center;
    gap: var(--s3);
    min-width: 0;
    pointer-events: none;
  }
  .wordmark {
    font-weight: 600;
    font-size: var(--t-sm);
    letter-spacing: -0.01em;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 320px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    padding: 2px 9px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--inset);
    color: var(--text-faint);
    font-size: var(--t-2xs);
    letter-spacing: 0.02em;
  }
  .pill.on {
    color: var(--text-dim);
    border-color: rgba(70, 211, 154, 0.3);
  }
  .pill .mono {
    color: var(--text);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--line-strong);
    flex: none;
  }
  .dot.live {
    background: var(--live);
    box-shadow: 0 0 7px var(--live);
  }

  /* Window controls — clickable (the rest of the bar drags). */
  .controls {
    display: flex;
    align-items: stretch;
    pointer-events: auto;
  }
  .ctl {
    display: grid;
    place-items: center;
    width: 46px;
    height: 38px;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--text-dim);
    transition: background 0.12s ease, color 0.12s ease;
  }
  .ctl:hover {
    background: var(--panel-2);
    color: var(--text);
  }
  .ctl:active {
    transform: none;
  }
  .ctl:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .ctl.close:hover {
    background: var(--danger);
    color: #1a0a0a;
  }
</style>
