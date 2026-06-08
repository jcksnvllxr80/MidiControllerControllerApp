<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import type { Request } from "../lib/protocol";
  import { humanizeError } from "../lib/errors";

  let json = "";
  let loading = false;
  let error = "";
  let count = 0;

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
      let n = 0;
      for (const g of groups) {
        out[g.key] = {};
        const names = (await request<string[]>({ op: g.list } as unknown as Request)) ?? [];
        for (const name of names) {
          out[g.key][name] = await request({ op: g.get, name } as unknown as Request);
          n += 1;
        }
      }
      json = JSON.stringify(out, null, 2);
      count = n;
    } catch (e) {
      error = humanizeError(e);
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
    <button class="primary" on:click={load} disabled={loading}>
      {loading ? "Loading…" : "Refresh"}
    </button>
  </div>
  {#if error}
    <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
  {:else if !loading && count === 0}
    <p class="muted empty-hint">No configuration on the device yet. Add sets, songs, and pedals in Configure.</p>
  {/if}
  <textarea class="viewer mono" readonly value={json} spellcheck="false" aria-label="Device configuration JSON"></textarea>
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
  .empty-hint {
    margin: 0;
    font-size: var(--t-sm);
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
