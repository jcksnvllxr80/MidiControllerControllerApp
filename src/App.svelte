<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { connection } from "./lib/stores";
  import {
    onConnectionStatus,
    fetchConnectionStatus,
    disconnectDevice,
  } from "./lib/transport";
  import { PROTOCOL_LABEL } from "./lib/protocol";
  import Connect from "./routes/Connect.svelte";
  import Control from "./routes/Control.svelte";
  import Configure from "./routes/Configure.svelte";
  import JsonView from "./routes/JsonView.svelte";

  type View = "control" | "configure" | "json";
  let view: View = "control";
  let unlisten: UnlistenFn | undefined;

  onMount(async () => {
    unlisten = await onConnectionStatus((s) => connection.set(s));
    try {
      connection.set(await fetchConnectionStatus());
    } catch {
      /* backend not ready yet; events will catch us up */
    }
  });
  onDestroy(() => unlisten?.());

  async function handleDisconnect() {
    try {
      await disconnectDevice();
    } catch {
      connection.set({ connected: false });
    }
  }
</script>

<main>
  {#if !$connection.connected}
    <Connect />
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
      <nav class="segmented">
        <button class:active={view === "control"} on:click={() => (view = "control")}>Control</button>
        <button class:active={view === "configure"} on:click={() => (view = "configure")}>Configure</button>
        <button class:active={view === "json"} on:click={() => (view = "json")}>JSON</button>
      </nav>
      <button class="disconnect" on:click={handleDisconnect}>Disconnect</button>
    </header>

    <section class="content">
      {#if view === "control"}
        <Control />
      {:else if view === "configure"}
        <Configure />
      {:else}
        <JsonView />
      {/if}
    </section>
  {/if}
</main>

<style>
  main {
    height: 100%;
    display: flex;
    flex-direction: column;
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
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--s6) var(--s5);
  }
</style>
