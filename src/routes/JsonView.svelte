<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import type { Request } from "../lib/protocol";

  let json = "";
  let loading = false;
  let error = "";

  const groups: { key: string; list: Request["op"]; get: Request["op"] }[] = [
    { key: "sets", list: "list_sets", get: "get_set" },
    { key: "songs", list: "list_songs", get: "get_song" },
    { key: "pedals", list: "list_pedals", get: "get_pedal" },
  ];

  async function load() {
    loading = true;
    error = "";
    try {
      const out: Record<string, Record<string, unknown>> = {};
      for (const g of groups) {
        out[g.key] = {};
        const names = (await request<string[]>({ op: g.list } as unknown as Request)) ?? [];
        for (const name of names) {
          out[g.key][name] = await request({ op: g.get, name } as unknown as Request);
        }
      }
      json = JSON.stringify(out, null, 2);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<div class="jsonview">
  <div class="toolbar">
    <span class="eyebrow">Device config · JSON</span>
    <span class="spacer"></span>
    {#if error}<span class="error">{error}</span>{/if}
    <button class="primary" on:click={load} disabled={loading}>
      {loading ? "Loading…" : "Refresh"}
    </button>
  </div>
  <textarea class="viewer mono" readonly value={json} spellcheck="false"></textarea>
</div>

<style>
  .jsonview {
    max-width: 940px;
    margin: 0 auto;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .spacer {
    flex: 1;
  }
  .viewer {
    flex: 1;
    min-height: 420px;
    background: var(--inset);
    border-radius: var(--r-lg);
    padding: var(--s4);
    color: var(--text-dim);
    font-size: var(--t-sm);
  }
</style>
