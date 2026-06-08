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
  let query = "";

  $: filtered = query.trim()
    ? names.filter((n) => n.toLowerCase().includes(query.trim().toLowerCase()))
    : names;
  $: editing = editorName !== "" || selected !== null;

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
    query = "";
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
  </div>

  <div class="panes">
    <aside class="list panel">
      <input
        class="search"
        type="text"
        bind:value={query}
        placeholder="Filter {kind}s…"
        aria-label="Filter {kind}s"
      />
      <div class="list-scroll">
        {#if loading}
          <p class="muted hint">Loading…</p>
        {:else if names.length === 0}
          <p class="muted hint">No {kind}s yet — create one.</p>
        {:else if filtered.length === 0}
          <p class="muted hint">No matches for “{query}”.</p>
        {/if}
        <ul>
          {#each filtered as name (name)}
            <li class:active={selected === name}>
              <button class="name" on:click={() => selectItem(name)}>{name}</button>
              <button
                class="del"
                title="Delete"
                on:click={() => remove(name)}
                aria-label="Delete {name}"
              >
                <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M4 7h16M9 7V5h6v2M7 7l1 12h8l1-12" />
                </svg>
              </button>
            </li>
          {/each}
        </ul>
      </div>
      <button class="add" on:click={newItem}>+ New {kind}</button>
    </aside>

    <section class="detail panel">
      {#if !editing}
        <div class="detail-empty">
          <p class="muted">Select a {kind} from the list, or create a new one.</p>
        </div>
      {:else}
        <div class="detail-head">
          <input class="title-input" bind:value={editorName} placeholder="{kind} name" aria-label="{kind} name" />
          <span class="type-chip mono">{kind}</span>
          <span class="grow"></span>
          <button class="primary" on:click={save} disabled={!editorJson.trim()}>Save {kind}</button>
        </div>
        <label class="jsonlabel">
          <span class="eyebrow">Config · JSON</span>
          <textarea bind:value={editorJson} rows="16" spellcheck="false"
            placeholder="Select an item, or click “New {kind}”."></textarea>
        </label>
        <span class="status-msg" class:ok={statusOk} class:error={!statusOk} role="status" aria-live="polite">
          {status}
        </span>
      {/if}
    </section>
  </div>
</div>

<style>
  .configure {
    max-width: 960px;
    margin: 0 auto;
  }
  .bar {
    display: flex;
    align-items: center;
    margin-bottom: var(--s4);
  }
  .panes {
    display: grid;
    grid-template-columns: 248px 1fr;
    gap: var(--s4);
    align-items: stretch;
    min-height: 460px;
  }
  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
  }

  /* Master list */
  .list {
    display: flex;
    flex-direction: column;
    padding: var(--s2);
    gap: var(--s2);
  }
  .search {
    font-size: var(--t-sm);
  }
  .list-scroll {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .hint {
    padding: var(--s2);
    font-size: var(--t-sm);
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
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
  .add {
    background: transparent;
    border: 1px dashed var(--line-strong);
    color: var(--text-dim);
    font-size: var(--t-sm);
  }
  .add:hover {
    color: var(--accent);
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }

  /* Detail */
  .detail {
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    padding: var(--s4);
  }
  .detail-empty {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--text-dim);
    font-size: var(--t-sm);
  }
  .detail-head {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .title-input {
    flex: 1;
    font-family: var(--font-ui);
    font-size: var(--t-lg);
    font-weight: 600;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--line);
    border-radius: 0;
    padding: var(--s2) 0;
  }
  .title-input:focus-visible {
    border-bottom-color: var(--accent);
    box-shadow: none;
  }
  .type-chip {
    font-size: var(--t-2xs);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-faint);
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 2px var(--s2);
  }
  .grow {
    flex: 1;
  }
  .jsonlabel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .status-msg:empty {
    display: none;
  }
  .status-msg {
    font-size: var(--t-sm);
  }
</style>
