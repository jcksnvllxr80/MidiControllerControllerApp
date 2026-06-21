<script lang="ts">
  import { afterUpdate } from "svelte";
  import { deviceLogs } from "../lib/stores";
  import { getLogDir } from "../lib/transport";

  let terminal: HTMLElement;
  let autoScroll = true;
  let logDir = "";

  getLogDir().then((d) => (logDir = d));

  afterUpdate(() => {
    if (autoScroll && terminal && typeof terminal.scrollTo === "function") {
      terminal.scrollTo({ top: terminal.scrollHeight });
    }
  });

  function clear() {
    deviceLogs.set([]);
  }

  function handleScroll() {
    if (!terminal) return;
    const atBottom = terminal.scrollHeight - terminal.scrollTop - terminal.clientHeight < 40;
    autoScroll = atBottom;
  }
</script>

<div class="logs-view">
  <div class="toolbar">
    <span class="eyebrow">Device Logs</span>
    <span class="count mono">{$deviceLogs.length} {$deviceLogs.length === 1 ? "line" : "lines"}</span>
    <label class="autoscroll-label">
      <input type="checkbox" bind:checked={autoScroll} />
      Auto-scroll
    </label>
    <button class="clear-btn" on:click={clear} disabled={$deviceLogs.length === 0}>Clear</button>
  </div>

  <div
    class="terminal"
    bind:this={terminal}
    on:scroll={handleScroll}
    role="log"
    aria-label="Device log output"
    aria-live="polite"
  >
    {#if $deviceLogs.length === 0}
      <div class="empty">
        No log output yet. Interact with the device to capture firmware logs.
      </div>
    {:else}
      {#each $deviceLogs as line, i (i)}
        <div class="line mono">{line}</div>
      {/each}
    {/if}
  </div>

  {#if logDir}
    <div class="log-path">
      <span class="log-path-label">Log files:</span>
      <span class="log-path-value mono">{logDir}</span>
    </div>
  {/if}
</div>

<style>
  .logs-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: var(--s3);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--s3);
    flex: none;
  }

  .toolbar .eyebrow {
    font-size: var(--t-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-faint);
    margin-right: auto;
  }

  .count {
    font-size: var(--t-xs);
    color: var(--text-faint);
  }

  .autoscroll-label {
    display: flex;
    align-items: center;
    gap: var(--s2);
    font-size: var(--t-sm);
    color: var(--text-dim);
    cursor: pointer;
    user-select: none;
  }

  .clear-btn {
    padding: 0.3rem 0.75rem;
    font-size: var(--t-sm);
  }

  .terminal {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    background: var(--inset);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .empty {
    color: var(--text-faint);
    font-size: var(--t-sm);
    text-align: center;
    margin: auto;
  }

  .line {
    font-size: var(--t-xs);
    color: var(--text-dim);
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .log-path {
    display: flex;
    align-items: baseline;
    gap: var(--s2);
    flex: none;
  }

  .log-path-label {
    font-size: var(--t-xs);
    color: var(--text-faint);
    white-space: nowrap;
  }

  .log-path-value {
    font-size: var(--t-xs);
    color: var(--text-faint);
    word-break: break-all;
  }
</style>
