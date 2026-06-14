<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import { humanizeError } from "../lib/errors";
  import type { WifiStatus } from "../lib/protocol";

  let ssid = "";
  let password = "";
  let status: WifiStatus | null = null;
  let busy = false;
  let error = "";
  let note = "";

  function apply(s: WifiStatus) {
    status = s;
    if (s.ssid && !ssid) ssid = s.ssid; // prefill from the device, don't clobber typing
  }

  async function refresh() {
    error = "";
    try {
      apply(await request<WifiStatus>({ op: "wifi_status" }));
    } catch (e) {
      error = humanizeError(e);
    }
  }

  async function save() {
    if (!ssid.trim()) {
      error = "Network name (SSID) is required.";
      return;
    }
    busy = true;
    error = "";
    note = "";
    try {
      const req = password
        ? { op: "wifi_set" as const, ssid: ssid.trim(), password }
        : { op: "wifi_set" as const, ssid: ssid.trim() };
      const s = await request<WifiStatus>(req);
      apply(s);
      password = "";
      note = s.connected ? `Connected — ${s.ip}` : "Saved. The controller is trying to join…";
    } catch (e) {
      error = humanizeError(e);
    } finally {
      busy = false;
    }
  }

  async function toggle() {
    busy = true;
    error = "";
    note = "";
    try {
      apply(await request<WifiStatus>({ op: "wifi_enable", on: !(status?.enabled ?? false) }));
    } catch (e) {
      error = humanizeError(e);
    } finally {
      busy = false;
    }
  }

  onMount(refresh);
</script>

<div class="wifi">
  <div class="surface">
    <header class="head">
      <span class="led" class:live={status?.connected}></span>
      <h2>Wi-Fi</h2>
      <span class="grow"></span>
      {#if status}
        <span class="state mono">
          {#if status.connected}{status.ssid} · {status.ip}
          {:else if status.enabled}Enabled · not connected
          {:else}Off{/if}
        </span>
      {/if}
    </header>

    <p class="muted hint">
      Set this once over USB — the controller then joins on boot, and the app
      discovers it over Wi-Fi (no cable).
    </p>

    {#if error}
      <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
    {/if}
    {#if note}
      <div class="notice warn" role="status"><span class="ic">✓</span><span>{note}</span></div>
    {/if}

    <label class="field">
      <span class="eyebrow">Network (SSID)</span>
      <input type="text" bind:value={ssid} placeholder="MyNetwork" autocomplete="off" spellcheck="false" />
    </label>
    <label class="field">
      <span class="eyebrow">Password</span>
      <input type="password" bind:value={password} placeholder="leave blank for an open network" autocomplete="off" />
    </label>

    <div class="actions">
      <button class="primary" on:click={save} disabled={busy || !ssid.trim()}>
        {busy ? "Working…" : "Save & Connect"}
      </button>
      <button on:click={refresh} disabled={busy}>Refresh</button>
      <span class="grow"></span>
      <button
        class="enable"
        class:on={status?.enabled}
        on:click={toggle}
        disabled={busy || !status}
        aria-pressed={status?.enabled ?? false}
      >
        <span class="dot"></span>Wi-Fi {status?.enabled ? "On" : "Off"}
      </button>
    </div>
  </div>
</div>

<style>
  .wifi {
    max-width: 520px;
    margin: 0 auto;
  }
  .surface {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s5);
    display: flex;
    flex-direction: column;
    gap: var(--s4);
  }
  .head {
    display: flex;
    align-items: center;
    gap: var(--s2);
  }
  .head h2 {
    font-size: var(--t-lg);
  }
  .head .led {
    background: var(--line-strong);
  }
  .head .led.live {
    background: var(--live);
    box-shadow: 0 0 8px var(--live);
  }
  .grow {
    flex: 1;
  }
  .state {
    font-size: var(--t-xs);
    color: var(--text-dim);
  }
  .hint {
    margin: 0;
    font-size: var(--t-sm);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--s1);
  }
  .field input {
    width: 100%;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--s2);
    border-top: 1px solid var(--line);
    padding-top: var(--s4);
  }
  .enable {
    display: inline-flex;
    align-items: center;
    gap: var(--s2);
    font-size: var(--t-sm);
    color: var(--text-dim);
  }
  .enable .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--line-strong);
  }
  .enable.on {
    color: var(--accent);
    border-color: var(--accent-line);
  }
  .enable.on .dot {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .notice .ic {
    font-weight: 700;
  }
</style>
