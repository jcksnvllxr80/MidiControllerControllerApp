<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import type { Request } from "../lib/protocol";
  import { humanizeError } from "../lib/errors";

  let json = "";
  let loading = false;
  let error = "";
  let counts = { sets: 0, songs: 0, pedals: 0 };
  let copied = false;

  const groups: { key: "sets" | "songs" | "pedals"; list: Request["op"]; get: Request["op"] }[] = [
    { key: "sets", list: "list_sets", get: "get_set" },
    { key: "songs", list: "list_songs", get: "get_song" },
    { key: "pedals", list: "list_pedals", get: "get_pedal" },
  ];

  $: total = counts.sets + counts.songs + counts.pedals;

  async function load() {
    loading = true;
    error = "";
    copied = false;
    try {
      const out: Record<string, Record<string, unknown>> = {};
      const next = { sets: 0, songs: 0, pedals: 0 };
      for (const g of groups) {
        out[g.key] = {};
        const names = (await request<string[]>({ op: g.list } as unknown as Request)) ?? [];
        for (const name of names) {
          out[g.key][name] = await request({ op: g.get, name } as unknown as Request);
        }
        next[g.key] = names.length;
      }
      json = JSON.stringify(out, null, 2);
      counts = next;
    } catch (e) {
      error = humanizeError(e);
    } finally {
      loading = false;
    }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(json);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      /* clipboard unavailable */
    }
  }

  onMount(load);
</script>

<div class="jsonview">
  <header class="head">
    <div class="title">
      <span class="eyebrow">Device configuration</span>
      <div class="chips mono">
        <span class="chip">{counts.sets} sets</span>
        <span class="chip">{counts.songs} songs</span>
        <span class="chip">{counts.pedals} pedals</span>
      </div>
    </div>
    <span class="grow"></span>
    <button class="ghost" on:click={copy} disabled={!json || loading}>
      {copied ? "Copied" : "Copy"}
    </button>
    <button class="primary" on:click={load} disabled={loading}>
      {loading ? "Loading…" : "Refresh"}
    </button>
  </header>

  {#if error}
    <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
  {:else if !loading && total === 0}
    <p class="muted empty-hint">No configuration on the device yet. Add sets, songs, and pedals in Configure.</p>
  {/if}

  <div class="well">
    <textarea class="viewer mono" readonly value={json} spellcheck="false" aria-label="Device configuration JSON"></textarea>
  </div>
</div>

<style>
  .jsonview {
    max-width: 960px;
    margin: 0 auto;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .head {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .title {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .chips {
    display: flex;
    gap: var(--s2);
  }
  .chip {
    font-size: var(--t-2xs);
    color: var(--text-dim);
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 2px var(--s2);
  }
  .grow {
    flex: 1;
  }
  .ghost {
    font-size: var(--t-sm);
    color: var(--text-dim);
    background: transparent;
  }
  .ghost:hover {
    color: var(--text);
  }
  .empty-hint {
    margin: 0;
    font-size: var(--t-sm);
  }
  .well {
    flex: 1;
    min-height: 0;
    display: flex;
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    background: var(--inset);
    overflow: hidden;
  }
  .viewer {
    flex: 1;
    min-height: 420px;
    resize: none;
    border: none;
    border-radius: 0;
    background: transparent;
    padding: var(--s4);
    color: var(--text-dim);
    font-size: var(--t-sm);
    line-height: 1.6;
  }
  .viewer:focus-visible {
    box-shadow: none;
  }
</style>
