<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import { ENTITY_OPS, type EntityKind, type Request } from "../lib/protocol";
  import { humanizeError } from "../lib/errors";

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
      setStatus(humanizeError(e), false);
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
      setStatus(humanizeError(e), false);
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
      setStatus(humanizeError(e), false);
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
      setStatus(humanizeError(e), false);
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
  <div class="bar">
    <div class="segmented">
      {#each ["set", "song", "pedal"] as k}
        <button
          class:active={kind === k}
          aria-current={kind === k ? "page" : undefined}
          on:click={() => switchKind(k as EntityKind)}
        >
          {k[0].toUpperCase() + k.slice(1)}s
        </button>
      {/each}
    </div>
    <button class="primary newbtn" on:click={newItem}>+ New {kind}</button>
  </div>

  <div class="panes">
    <aside class="list panel">
      <span class="eyebrow">{kind}s</span>
      {#if loading}
        <p class="muted hint">Loading…</p>
      {:else if names.length === 0}
        <div class="list-empty">
          <p class="muted">No {kind}s yet.</p>
          <button class="ghost" on:click={newItem}>+ New {kind}</button>
        </div>
      {/if}
      <ul>
        {#each names as name (name)}
          <li class:active={selected === name}>
            <button class="name" on:click={() => selectItem(name)}>{name}</button>
            <button class="del" title="Delete" on:click={() => remove(name)} aria-label="Delete {name}">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16M9 7V5h6v2M7 7l1 12h8l1-12" />
              </svg>
            </button>
          </li>
        {/each}
      </ul>
    </aside>

    <section class="editor panel">
      <label>
        <span class="eyebrow">Name</span>
        <input type="text" bind:value={editorName} placeholder="{kind} name" />
      </label>
      <label class="jsonlabel">
        <span class="eyebrow">Config · JSON</span>
        <textarea bind:value={editorJson} rows="18" spellcheck="false"
          placeholder="Select an item, or click “New {kind}”."></textarea>
      </label>
      <div class="actions">
        <button class="primary" on:click={save} disabled={!editorJson.trim()}>Save {kind}</button>
        <span class="status-msg" class:ok={statusOk} class:error={!statusOk} role="status" aria-live="polite">
          {status}
        </span>
      </div>
    </section>
  </div>
</div>

<style>
  .configure {
    max-width: 940px;
    margin: 0 auto;
  }
  .bar {
    display: flex;
    align-items: center;
    margin-bottom: var(--s4);
  }
  .newbtn {
    margin-left: auto;
  }
  .panes {
    display: grid;
    grid-template-columns: 232px 1fr;
    gap: var(--s4);
    align-items: start;
  }
  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s3);
  }
  .list {
    min-height: 240px;
  }
  .list .eyebrow {
    display: block;
    padding: var(--s1) var(--s2) var(--s2);
  }
  .hint {
    padding: var(--s2);
    font-size: var(--t-sm);
  }
  .list-empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--s2);
    padding: var(--s2);
  }
  .ghost {
    background: transparent;
    border: 1px dashed var(--line-strong);
    color: var(--text-dim);
    font-size: var(--t-sm);
    padding: 0.4rem 0.6rem;
  }
  .ghost:hover {
    color: var(--accent);
    border-color: var(--accent-line);
  }
  .status-msg:empty {
    display: none;
  }
  .list ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list li {
    display: flex;
    align-items: center;
    gap: var(--s1);
    border-radius: var(--r-sm);
  }
  .list li.active {
    background: var(--accent-soft);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .list .name {
    flex: 1;
    text-align: left;
    background: transparent;
    border: none;
    padding: 0.5rem 0.55rem;
    border-radius: var(--r-sm);
    color: var(--text);
  }
  .list li.active .name {
    color: var(--accent);
  }
  .list .name:hover {
    color: var(--accent);
    background: transparent;
  }
  .list .del {
    display: grid;
    place-items: center;
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0.3rem 0.4rem;
  }
  .list .del:hover {
    color: var(--danger);
    background: transparent;
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    padding: var(--s4);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .jsonlabel {
    flex: 1;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--s4);
    font-size: var(--t-sm);
  }
</style>
