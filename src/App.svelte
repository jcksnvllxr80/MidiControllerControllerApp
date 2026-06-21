<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { connection, connectionError, appendDeviceLog, deviceLogs } from "./lib/stores";
  import {
    onConnectionStatus,
    onDeviceLog,
    fetchConnectionStatus,
    disconnectDevice,
    sendRequest,
  } from "./lib/transport";
  import Connect from "./routes/Connect.svelte";
  import Control from "./routes/Control.svelte";
  import Configure from "./routes/Configure.svelte";
  import JsonView from "./routes/JsonView.svelte";
  import Wifi from "./routes/Wifi.svelte";
  import Firmware from "./routes/Firmware.svelte";
  import Appearance from "./routes/Appearance.svelte";
  import Logs from "./routes/Logs.svelte";
  import TitleBar from "./TitleBar.svelte";
  import Sidebar from "./Sidebar.svelte";
  import { appVersion } from "./lib/app";

  type View = "control" | "configure" | "json" | "wifi" | "firmware" | "logs" | "appearance";
  let view: View = "control";
  let unlisten: UnlistenFn | undefined;
  let unlistenLog: UnlistenFn | undefined;
  let heartbeat: ReturnType<typeof setInterval> | undefined;
  let version = "";

  onMount(async () => {
    appVersion().then((v) => (version = v));
    unlisten = await onConnectionStatus((s) => {
      if (s.connected) deviceLogs.set([]);
      connection.set(s);
    });
    unlistenLog = await onDeviceLog(appendDeviceLog);
    try {
      connection.set(await fetchConnectionStatus());
    } catch {
      /* backend not ready yet; events will catch us up */
    }
  });
  onDestroy(() => {
    unlisten?.();
    unlistenLog?.();
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

<div class="shell">
  <TitleBar />
  <div class="app">
  {#if !$connection.connected}
    <main><Connect /></main>
  {:else}
    <div class="workspace">
      <Sidebar {view} onSelect={(v) => (view = v as View)} onDisconnect={handleDisconnect} />
      <main class="content">
        {#if view === "control"}
          <Control />
        {:else if view === "configure"}
          <Configure />
        {:else if view === "json"}
          <JsonView />
        {:else if view === "wifi"}
          <Wifi />
        {:else if view === "firmware"}
          <Firmware />
        {:else if view === "logs"}
          <Logs />
        {:else}
          <Appearance />
        {/if}
      </main>
    </div>
  {/if}
  </div>
  {#if version}<span class="app-version mono">v{version}</span>{/if}
</div>

<style>
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .app {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .app > main {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .workspace {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--s6) var(--s5);
  }
  /* Inconspicuous version readout, pinned to the bottom-right of the window. */
  .app-version {
    position: fixed;
    right: 10px;
    bottom: 6px;
    font-size: var(--t-2xs);
    color: var(--text-faint);
    opacity: 0.55;
    pointer-events: none;
    user-select: none;
    z-index: 5;
  }
</style>
