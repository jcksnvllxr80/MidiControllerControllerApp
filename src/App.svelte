<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { connection, connectionError } from "./lib/stores";
  import {
    onConnectionStatus,
    fetchConnectionStatus,
    disconnectDevice,
    sendRequest,
  } from "./lib/transport";
  import { PROTOCOL_LABEL } from "./lib/protocol";
  import Connect from "./routes/Connect.svelte";
  import Control from "./routes/Control.svelte";
  import Configure from "./routes/Configure.svelte";
  import JsonView from "./routes/JsonView.svelte";
  import Wifi from "./routes/Wifi.svelte";
  import Firmware from "./routes/Firmware.svelte";

  type View = "control" | "configure" | "json" | "wifi" | "firmware";
  let view: View = "control";
  let unlisten: UnlistenFn | undefined;
  let heartbeat: ReturnType<typeof setInterval> | undefined;

  onMount(async () => {
    unlisten = await onConnectionStatus((s) => connection.set(s));
    try {
      connection.set(await fetchConnectionStatus());
    } catch {
      /* backend not ready yet; events will catch us up */
    }
  });
  onDestroy(() => {
    unlisten?.();
    stopHeartbeat();
  });

  // While connected, ping the device so a silent unplug surfaces as a visible
  // "lost connection" instead of stale data. Clear any prior loss on connect.
  $: manageHeartbeat($connection.connected);
  function manageHeartbeat(connected: boolean) {
    if (connected) {
      connectionError.set("");
      if (!heartbeat) heartbeat = setInterval(pingOnce, 5000);
    } else {
      stopHeartbeat();
    }
  }
  function stopHeartbeat() {
    if (heartbeat) {
      clearInterval(heartbeat);
      heartbeat = undefined;
    }
  }
  async function pingOnce() {
    try {
      const resp = await sendRequest({ op: "ping" });
      if (!resp.ok) throw new Error(resp.error ?? "ping failed");
    } catch {
      const name = $connection.identity?.name ?? $connection.device?.name ?? "the controller";
      stopHeartbeat();
      connectionError.set(`Lost connection to ${name}.`);
      try {
        await disconnectDevice();
      } catch {
        connection.set({ connected: false });
      }
    }
  }

  async function handleDisconnect() {
    try {
      await disconnectDevice();
    } catch {
      connection.set({ connected: false });
    }
  }
</script>

<div class="app">
  {#if !$connection.connected}
    <main><Connect /></main>
  {:else}
    <header class="topbar">
      <div class="device">
        <span class="led live" title="Connected"></span>
        <strong>{$connection.identity?.name ?? $connection.device?.name ?? "Connected"}</strong>
        <span class="device-meta mono">
          {PROTOCOL_LABEL[$connection.device?.protocol ?? "mock"]}
          {#if $connection.identity?.firmware}· {$connection.identity.firmware}{/if}
        </span>
      </div>
      <nav class="segmented" aria-label="Views">
        <button
          class:active={view === "control"}
          aria-current={view === "control" ? "page" : undefined}
          on:click={() => (view = "control")}>Control</button
        >
        <button
          class:active={view === "configure"}
          aria-current={view === "configure" ? "page" : undefined}
          on:click={() => (view = "configure")}>Configure</button
        >
        <button
          class:active={view === "json"}
          aria-current={view === "json" ? "page" : undefined}
          on:click={() => (view = "json")}>JSON</button
        >
        <button
          class:active={view === "wifi"}
          aria-current={view === "wifi" ? "page" : undefined}
          on:click={() => (view = "wifi")}>Wi-Fi</button
        >
        <button
          class:active={view === "firmware"}
          aria-current={view === "firmware" ? "page" : undefined}
          on:click={() => (view = "firmware")}>Firmware</button
        >
      </nav>
      <button class="disconnect" on:click={handleDisconnect}>Disconnect</button>
    </header>

    <main class="content">
      {#if view === "control"}
        <Control />
      {:else if view === "configure"}
        <Configure />
      {:else if view === "json"}
        <JsonView />
      {:else if view === "wifi"}
        <Wifi />
      {:else}
        <Firmware />
      {/if}
    </main>
  {/if}
</div>

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .app > main {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--s4);
    padding: var(--s3) var(--s4);
    border-bottom: 1px solid var(--line);
    background: var(--panel);
  }
  .device {
    display: flex;
    align-items: center;
    gap: var(--s2);
    min-width: 0;
  }
  .device strong {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 600;
  }
  .device-meta {
    font-size: var(--t-xs);
    color: var(--text-dim);
    white-space: nowrap;
  }
  nav {
    margin-left: auto;
  }
  .disconnect {
    font-size: var(--t-sm);
    color: var(--text-dim);
  }
  .disconnect:hover {
    border-color: var(--danger);
    color: var(--danger);
    background: var(--panel-2);
  }
  .content {
    padding: var(--s6) var(--s5);
  }
</style>
