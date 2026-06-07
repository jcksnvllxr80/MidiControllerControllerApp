<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import { ENTITY_OPS, type EntityKind, type Request } from "../lib/protocol";

  let kind: EntityKind = "set";
  let names: string[] = [];
  let selected: string | null = null;
  let editorName = "";
  let editorJson = "";
  let status = "";
  let statusOk = false;
  let loading = false;

  function template(k: EntityKind): unknown {
    if (k === "set") return { name: "", songs: [] };
    if (k === "song") return { name: "", tempo: 120, parts: {} };
    return { name: "", presets: [], params: [] };
  }

  async function refresh() {
    loading = true;
    setStatus("", false);
    try {
      names = (await request<string[]>({ op: ENTITY_OPS[kind].list } as unknown as Request)) ?? [];
    } catch (e) {
      names = [];
      setStatus(String(e), false);
    } finally {
      loading = false;
    }
  }

  async function selectItem(name: string) {
    selected = name;
    setStatus("", false);
    try {
      const data = await request({ op: ENTITY_OPS[kind].get, name } as unknown as Request);
      editorName = name;
      editorJson = JSON.stringify(data, null, 2);
    } catch (e) {
      setStatus(String(e), false);
    }
  }

  function newItem() {
    selected = null;
    editorName = `new-${kind}`;
    editorJson = JSON.stringify(template(kind), null, 2);
    setStatus("", false);
  }

  async function save() {
    let data: unknown;
    try {
      data = JSON.parse(editorJson);
    } catch {
      setStatus("Invalid JSON — fix and try again.", false);
      return;
    }
    if (!editorName.trim()) {
      setStatus("Name is required.", false);
      return;
    }
    try {
      await request({ op: ENTITY_OPS[kind].write, name: editorName, data } as unknown as Request);
      setStatus(`Saved “${editorName}”.`, true);
      await refresh();
      selected = editorName;
    } catch (e) {
      setStatus(String(e), false);
    }
  }

  async function remove(name: string) {
    try {
      await request({ op: ENTITY_OPS[kind].del, name } as unknown as Request);
      if (selected === name) {
        selected = null;
        editorName = "";
        editorJson = "";
      }
      setStatus(`Deleted “${name}”.`, true);
      await refresh();
    } catch (e) {
      setStatus(String(e), false);
    }
  }

  function switchKind(k: EntityKind) {
    if (k === kind) return;
    kind = k;
    selected = null;
    editorName = "";
    editorJson = "";
    refresh();
  }

  function setStatus(msg: string, ok: boolean) {
    status = msg;
    statusOk = ok;
  }

  onMount(refresh);
</script>

<div class="configure">
  <div class="tabs">
    {#each ["set", "song", "pedal"] as k}
      <button class:active={kind === k} on:click={() => switchKind(k as EntityKind)}>
        {k[0].toUpperCase() + k.slice(1)}s
      </button>
    {/each}
    <button class="primary newbtn" on:click={newItem}>+ New {kind}</button>
  </div>

  <div class="panes">
    <aside class="list">
      {#if loading}
        <p class="muted">Loading…</p>
      {:else if names.length === 0}
        <p class="muted">No {kind}s yet.</p>
      {/if}
      <ul>
        {#each names as name (name)}
          <li class:active={selected === name}>
            <button class="name" on:click={() => selectItem(name)}>{name}</button>
            <button class="del" title="Delete" on:click={() => remove(name)}>🗑</button>
          </li>
        {/each}
      </ul>
    </aside>

    <section class="editor">
      <label>
        Name
        <input type="text" bind:value={editorName} placeholder="{kind} name" />
      </label>
      <label class="jsonlabel">
        Config (JSON)
        <textarea bind:value={editorJson} rows="18" spellcheck="false"
          placeholder="Select an item, or click “New {kind}”."></textarea>
      </label>
      <div class="actions">
        <button class="primary" on:click={save} disabled={!editorJson.trim()}>Save {kind}</button>
        {#if status}
          <span class:ok={statusOk} class:error={!statusOk}>{status}</span>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .configure {
    max-width: 920px;
    margin: 0 auto;
  }
  .tabs {
    display: flex;
    gap: 0.4rem;
    margin-bottom: 1rem;
  }
  .tabs button.active {
    border-color: var(--accent);
    background: var(--bg-elev-2);
  }
  .newbtn {
    margin-left: auto;
  }
  .panes {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 1rem;
    align-items: start;
  }
  .list {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.5rem;
    min-height: 200px;
  }
  .list ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list li {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    border-radius: 8px;
  }
  .list li.active {
    background: var(--bg-elev-2);
  }
  .list .name {
    flex: 1;
    text-align: left;
    background: transparent;
    border: none;
    padding: 0.5rem 0.5rem;
    border-radius: 8px;
  }
  .list .name:hover {
    color: var(--accent);
  }
  .list .del {
    background: transparent;
    border: none;
    opacity: 0.5;
    padding: 0.3rem 0.4rem;
  }
  .list .del:hover {
    opacity: 1;
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--text-dim);
  }
  .jsonlabel {
    flex: 1;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }
</style>
