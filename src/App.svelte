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
        <span class="dot"></span>
        <strong>{$connection.identity?.name ?? $connection.device?.name ?? "Connected"}</strong>
        <span class="muted tag">
          {PROTOCOL_LABEL[$connection.device?.protocol ?? "mock"]}
          {#if $connection.identity?.firmware}· fw {$connection.identity.firmware}{/if}
        </span>
      </div>
      <nav>
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
    gap: 1rem;
    padding: 0.65rem 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev);
  }
  .device {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .device strong {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tag {
    font-size: 0.8rem;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--good);
    box-shadow: 0 0 8px var(--good);
    flex: none;
  }
  nav {
    display: flex;
    gap: 0.4rem;
    margin-left: auto;
  }
  nav button.active {
    border-color: var(--accent);
    background: var(--bg-elev-2);
  }
  .disconnect {
    border-color: var(--border);
  }
  .disconnect:hover {
    border-color: var(--bad);
    color: var(--bad);
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 1.25rem;
  }
</style>
