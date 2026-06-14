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
  import Connect from "./routes/Connect.svelte";
  import Control from "./routes/Control.svelte";
  import Configure from "./routes/Configure.svelte";
  import JsonView from "./routes/JsonView.svelte";
  import Wifi from "./routes/Wifi.svelte";
  import Firmware from "./routes/Firmware.svelte";
  import TitleBar from "./TitleBar.svelte";
  import Sidebar from "./Sidebar.svelte";

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
        {:else}
          <Firmware />
        {/if}
      </main>
    </div>
  {/if}
  </div>
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
</style>
