<script lang="ts">
  import { onMount } from "svelte";
  import { request } from "../lib/transport";
  import { ENTITY_OPS, type EntityKind, type Request } from "../lib/protocol";
  import { humanizeError } from "../lib/errors";
  import { presetOptions, presetChoices } from "../lib/presets";

  // ── Entity rail definition (the vertical icon nav) ──────────────────────
  type KindDef = { key: EntityKind; label: string; singular: string };
  const KINDS: KindDef[] = [
    { key: "set", label: "Sets", singular: "set" },
    { key: "song", label: "Songs", singular: "song" },
    { key: "pedal", label: "Pedals", singular: "pedal" },
  ];

  let kind: EntityKind = "set";
  $: singular = KINDS.find((k) => k.key === kind)!.singular;

  let names: string[] = [];
  let selected: string | null = null;
  let editorName = "";
  let editor: any = null; // the structured object being edited
  let loading = false;
  let query = "";
  let status = "";
  let statusOk = false;

  // Aux data for the dropdowns (lazy-loaded, cached per session)
  let allSongs: string[] = []; // choices for "add song to set"
  let allPedals: string[] = []; // choices for "add pedal to part"
  let addSongChoice = "";
  let newPartName = "";
  let addPedalChoice: Record<string, string> = {}; // per part name
  let pedalDefs: Record<string, any> = {}; // pedal name -> definition (for preset options)

  // Advanced raw-JSON escape hatch
  let showRaw = false;
  let rawJson = "";

  $: filtered = query.trim()
    ? names.filter((n) => n.toLowerCase().includes(query.trim().toLowerCase()))
    : names;
  $: editing = editor !== null;

  function setStatus(msg: string, ok: boolean) {
    status = msg;
    statusOk = ok;
  }

  // Typed Object.entries so `#each` destructures to `any`, not `unknown`.
  function entries(obj: any): [string, any][] {
    return Object.entries(obj ?? {}) as [string, any][];
  }

  async function loadPedalDefs() {
    const missing = allPedals.filter((p) => !(p in pedalDefs));
    if (missing.length === 0) return;
    const next = { ...pedalDefs };
    for (const p of missing) {
      try {
        next[p] = await request({ op: ENTITY_OPS.pedal.get, name: p } as unknown as Request);
      } catch {
        next[p] = null;
      }
    }
    pedalDefs = next; // reassign → preset inputs upgrade to dropdowns
  }

  function template(k: EntityKind): any {
    if (k === "set") return { name: "", songs: [] };
    if (k === "song") return { name: "", tempo: 120, parts: {} };
    return { name: "", "Set Preset": {}, Parameters: {} };
  }

  async function listOf(k: EntityKind): Promise<string[]> {
    return (await request<string[]>({ op: ENTITY_OPS[k].list } as unknown as Request)) ?? [];
  }

  async function refresh() {
    loading = true;
    setStatus("", false);
    try {
      names = await listOf(kind);
    } catch (e) {
      names = [];
      setStatus(humanizeError(e), false);
    } finally {
      loading = false;
    }
  }

  // Pull the choice lists the structured editors need, once.
  async function ensureAux() {
    try {
      if (kind === "set" && allSongs.length === 0) allSongs = await listOf("song");
      if (kind === "song") {
        if (allPedals.length === 0) allPedals = await listOf("pedal");
        await loadPedalDefs(); // preset dropdowns need each pedal's definition
      }
    } catch {
      /* choices stay empty; editor still usable */
    }
  }

  async function selectItem(name: string) {
    selected = name;
    setStatus("", false);
    try {
      const data = await request({ op: ENTITY_OPS[kind].get, name } as unknown as Request);
      editor = data ?? template(kind);
      editorName = name;
      showRaw = false;
      ensureAux();
    } catch (e) {
      setStatus(humanizeError(e), false);
    }
  }

  function newItem() {
    selected = null;
    editor = template(kind);
    editorName = `New ${singular}`;
    showRaw = false;
    setStatus("", false);
    ensureAux();
  }

  function closeEditor() {
    editor = null;
    selected = null;
    editorName = "";
    showRaw = false;
  }

  async function save() {
    if (!editor) return;
    if (!editorName.trim()) {
      setStatus("Name is required.", false);
      return;
    }
    editor.name = editorName.trim();
    try {
      await request({
        op: ENTITY_OPS[kind].write,
        name: editorName.trim(),
        data: editor,
      } as unknown as Request);
      setStatus(`Saved “${editorName.trim()}”.`, true);
      await refresh();
      selected = editorName.trim();
    } catch (e) {
      setStatus(humanizeError(e), false);
    }
  }

  async function remove(name: string) {
    try {
      await request({ op: ENTITY_OPS[kind].del, name } as unknown as Request);
      if (selected === name) closeEditor();
      setStatus(`Deleted “${name}”.`, true);
      await refresh();
    } catch (e) {
      setStatus(humanizeError(e), false);
    }
  }

  function switchKind(k: EntityKind) {
    if (k === kind) return;
    kind = k;
    query = "";
    closeEditor();
    refresh();
  }

  // ── Set editor mutators ────────────────────────────────────────────────
  $: setSongChoices = editor && kind === "set"
    ? allSongs.filter((s) => !(editor.songs ?? []).includes(s))
    : [];
  function addSong() {
    if (!addSongChoice) return;
    editor.songs = [...(editor.songs ?? []), addSongChoice];
    addSongChoice = "";
    editor = editor;
  }
  function removeSong(i: number) {
    editor.songs = editor.songs.filter((_: string, idx: number) => idx !== i);
    editor = editor;
  }
  function moveSong(i: number, dir: -1 | 1) {
    const j = i + dir;
    const a = editor.songs;
    if (j < 0 || j >= a.length) return;
    [a[i], a[j]] = [a[j], a[i]];
    editor = editor;
  }

  // ── Song editor mutators ───────────────────────────────────────────────
  function addPart() {
    const n = newPartName.trim();
    if (!n) return;
    if (!editor.parts) editor.parts = {};
    if (editor.parts[n]) {
      setStatus(`Part “${n}” already exists.`, false);
      return;
    }
    editor.parts[n] = { position: Object.keys(editor.parts).length + 1, pedals: {} };
    newPartName = "";
    editor = editor;
  }
  function removePart(name: string) {
    delete editor.parts[name];
    editor = editor;
  }
  function pedalsLeft(partName: string): string[] {
    const used = Object.keys(editor.parts[partName].pedals ?? {});
    return allPedals.filter((p) => !used.includes(p));
  }
  function addPedal(partName: string) {
    const choice = addPedalChoice[partName];
    if (!choice) return;
    const part = editor.parts[partName];
    if (!part.pedals) part.pedals = {};
    const opts = presetOptions(pedalDefs[choice]);
    part.pedals[choice] = { engaged: true, preset: opts && opts.length ? opts[0].value : 0 };
    addPedalChoice[partName] = "";
    editor = editor;
  }
  function removePedal(partName: string, pedal: string) {
    delete editor.parts[partName].pedals[pedal];
    editor = editor;
  }
  function setPreset(partName: string, pedal: string, raw: string) {
    const v = raw.trim();
    editor.parts[partName].pedals[pedal].preset = v !== "" && !isNaN(Number(v)) ? Number(v) : v;
    editor = editor;
  }

  // ── Advanced raw JSON ──────────────────────────────────────────────────
  function toggleRaw() {
    showRaw = !showRaw;
    if (showRaw) rawJson = JSON.stringify(editor, null, 2);
  }
  function applyRaw() {
    let parsed: any;
    try {
      parsed = JSON.parse(rawJson);
    } catch {
      setStatus("Invalid JSON — fix and try again.", false);
      return;
    }
    editor = parsed;
    if (typeof editor.name === "string" && editor.name) editorName = editor.name;
    setStatus("Applied JSON to the editor. Review, then Save.", true);
  }

  onMount(refresh);
</script>

<div class="workbench">
  <!-- ── Vertical icon rail ── -->
  <nav class="rail" aria-label="Entity type">
    {#each KINDS as k}
      <button
        class="rail-btn"
        class:active={kind === k.key}
        aria-current={kind === k.key ? "page" : undefined}
        title={k.label}
        on:click={() => switchKind(k.key)}
      >
        <span class="rail-ic" aria-hidden="true">
          {#if k.key === "set"}
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 3 3 7.5 12 12l9-4.5L12 3Z" /><path d="M3 12l9 4.5L21 12" /><path d="M3 16.5 12 21l9-4.5" />
            </svg>
          {:else if k.key === "song"}
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 17V5l11-2v12" /><circle cx="6" cy="17" r="3" /><circle cx="17" cy="15" r="3" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="4" y="3" width="16" height="18" rx="2" /><circle cx="12" cy="9" r="2.4" /><path d="M8 16h8" />
            </svg>
          {/if}
        </span>
        <span class="rail-label">{k.label}</span>
      </button>
    {/each}
  </nav>

  <!-- ── Master list ── -->
  <aside class="list-col">
    <header class="list-head">
      <h2>{KINDS.find((k) => k.key === kind)!.label}</h2>
      <span class="count mono">{names.length}</span>
    </header>
    <div class="list-search">
      <input type="text" bind:value={query} placeholder="Filter {singular}s…" aria-label="Filter {singular}s" />
    </div>
    <div class="list-scroll">
      {#if loading}
        <p class="muted hint">Loading…</p>
      {:else if names.length === 0}
        <p class="muted hint">No {singular}s yet.</p>
      {:else if filtered.length === 0}
        <p class="muted hint">No matches for “{query}”.</p>
      {/if}
      <ul>
        {#each filtered as name (name)}
          <li class:active={selected === name}>
            <button class="row-name" on:click={() => selectItem(name)}>{name}</button>
            <button class="row-del" title="Delete" aria-label="Delete {name}" on:click={() => remove(name)}>
              <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16M9 7V5h6v2M7 7l1 12h8l1-12" />
              </svg>
            </button>
          </li>
        {/each}
      </ul>
    </div>
    <button class="list-add" on:click={newItem}>+ New {singular}</button>
  </aside>

  <!-- ── Detail editor ── -->
  <section class="detail-col">
    {#if !editing}
      <div class="detail-empty">
        <p class="muted">Select a {singular} from the list, or create a new one.</p>
      </div>
    {:else}
      <header class="detail-head">
        <input class="title-input" bind:value={editorName} placeholder="{singular} name" aria-label="{singular} name" />
        <span class="grow"></span>
        <button class="ghost" on:click={closeEditor} aria-label="Close editor">Close</button>
        <button class="primary" on:click={save}>Save {singular}</button>
      </header>

      {#if status}
        <p class="status" class:ok={statusOk} class:error={!statusOk} role="status" aria-live="polite">{status}</p>
      {/if}

      <!-- ── SET editor ── -->
      {#if kind === "set"}
        <div class="section">
          <p class="eyebrow">Songs in this set · {(editor.songs ?? []).length}</p>
          {#if (editor.songs ?? []).length === 0}
            <p class="muted hint">No songs yet — add one below.</p>
          {/if}
          <ol class="chips">
            {#each editor.songs ?? [] as song, i (song + i)}
              <li class="chip">
                <span class="chip-idx mono">{i + 1}</span>
                <span class="chip-name">{song}</span>
                <span class="chip-actions">
                  <button class="icon" title="Move up" aria-label="Move {song} up" disabled={i === 0} on:click={() => moveSong(i, -1)}>↑</button>
                  <button class="icon" title="Move down" aria-label="Move {song} down" disabled={i === editor.songs.length - 1} on:click={() => moveSong(i, 1)}>↓</button>
                  <button class="icon danger" title="Remove" aria-label="Remove {song}" on:click={() => removeSong(i)}>×</button>
                </span>
              </li>
            {/each}
          </ol>
          <div class="adder">
            <select bind:value={addSongChoice} aria-label="Add song">
              <option value="" disabled selected>Add a song…</option>
              {#each setSongChoices as s}<option value={s}>{s}</option>{/each}
            </select>
            <button on:click={addSong} disabled={!addSongChoice}>Add</button>
          </div>
        </div>

      <!-- ── SONG editor ── -->
      {:else if kind === "song"}
        <div class="field-row">
          <label class="field">
            <span class="eyebrow">Tempo (BPM)</span>
            <input type="number" min="20" max="400" bind:value={editor.tempo} />
          </label>
        </div>

        <div class="section">
          <p class="eyebrow">Parts · {Object.keys(editor.parts ?? {}).length}</p>
          {#each entries(editor.parts) as [partName, part] (partName)}
            <div class="part">
              <div class="part-head">
                <strong>{partName}</strong>
                <span class="count mono">{Object.keys(part.pedals ?? {}).length} pedals</span>
                <span class="grow"></span>
                <button class="icon danger" title="Remove part" aria-label="Remove part {partName}" on:click={() => removePart(partName)}>×</button>
              </div>
              <ul class="pedals">
                {#each entries(part.pedals) as [pedalName, pd] (pedalName)}
                  {@const opts = presetChoices(pedalDefs, pedalName, pd.preset)}
                  <li class="pedal">
                    <label class="toggle" title="Engaged">
                      <input type="checkbox" bind:checked={pd.engaged} />
                      <span class="toggle-name">{pedalName}</span>
                    </label>
                    <span class="grow"></span>
                    <label class="preset">
                      <span class="eyebrow">preset</span>
                      {#if opts}
                        <select
                          class="preset-input"
                          aria-label="{pedalName} preset"
                          on:change={(e) => setPreset(partName, pedalName, e.currentTarget.value)}
                        >
                          {#each opts as o (o.value)}
                            <option value={String(o.value)} selected={String(o.value) === String(pd.preset ?? "")}>{o.label}</option>
                          {/each}
                        </select>
                      {:else}
                        <input
                          class="preset-input"
                          aria-label="{pedalName} preset"
                          value={pd.preset ?? ""}
                          on:input={(e) => setPreset(partName, pedalName, e.currentTarget.value)}
                        />
                      {/if}
                    </label>
                    <button class="icon danger" title="Remove pedal" aria-label="Remove {pedalName}" on:click={() => removePedal(partName, pedalName)}>×</button>
                  </li>
                {/each}
              </ul>
              <div class="adder">
                <select bind:value={addPedalChoice[partName]} aria-label="Add pedal to {partName}">
                  <option value="" disabled selected>Add a pedal…</option>
                  {#each pedalsLeft(partName) as p}<option value={p}>{p}</option>{/each}
                </select>
                <button on:click={() => addPedal(partName)} disabled={!addPedalChoice[partName]}>Add</button>
              </div>
            </div>
          {/each}
          <div class="adder">
            <input type="text" bind:value={newPartName} placeholder="New part name (e.g. Chorus)" aria-label="New part name" />
            <button on:click={addPart} disabled={!newPartName.trim()}>Add part</button>
          </div>
        </div>

      <!-- ── PEDAL editor (device spec: structured read + raw) ── -->
      {:else}
        <div class="section">
          <p class="eyebrow">Pedal capabilities</p>
          <p class="muted hint">
            Pedal definitions describe the device (presets, parameters). Edit the spec
            below via Advanced · Raw JSON.
          </p>
          <ul class="spec">
            {#each Object.keys(editor).filter((k) => k !== "name") as group}
              <li><span class="spec-key">{group}</span>
                <span class="count mono">
                  {typeof editor[group] === "object" && editor[group] ? Object.keys(editor[group]).length : ""}
                </span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- ── Advanced raw JSON (escape hatch for power edits) ── -->
      <div class="advanced">
        <button class="disclose" on:click={toggleRaw} aria-expanded={showRaw}>
          <span class="caret" class:open={showRaw}>▸</span> Advanced · Raw JSON
        </button>
        {#if showRaw}
          <textarea bind:value={rawJson} rows="12" spellcheck="false" aria-label="Raw JSON" placeholder="Raw JSON"></textarea>
          <button class="ghost" on:click={applyRaw}>Apply JSON</button>
        {/if}
      </div>
    {/if}
  </section>
</div>

<style>
  .workbench {
    display: grid;
    grid-template-columns: 84px 268px 1fr;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    overflow: hidden;
    min-height: 560px;
    max-width: 1120px;
    margin: 0 auto;
  }

  /* ── Rail ── */
  .rail {
    display: flex;
    flex-direction: column;
    gap: var(--s1);
    padding: var(--s3) var(--s2);
    background: var(--bg);
    border-right: 1px solid var(--line);
  }
  .rail-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: var(--s3) var(--s1);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-md);
    color: var(--text-dim);
  }
  .rail-btn:hover {
    background: var(--panel-2);
    color: var(--text);
    border-color: transparent;
  }
  .rail-btn.active {
    color: var(--accent);
    background: var(--accent-soft);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .rail-ic {
    display: grid;
    place-items: center;
  }
  .rail-label {
    font-size: var(--t-2xs);
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  /* ── List column ── */
  .list-col {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--line);
    min-height: 0;
  }
  .list-head {
    display: flex;
    align-items: baseline;
    gap: var(--s2);
    padding: var(--s4) var(--s4) var(--s2);
  }
  .list-head h2 {
    font-size: var(--t-lg);
  }
  .count {
    font-size: var(--t-2xs);
    color: var(--text-faint);
  }
  .list-search {
    padding: 0 var(--s3) var(--s2);
  }
  .list-search input {
    width: 100%;
    font-size: var(--t-sm);
  }
  .list-scroll {
    flex: 1;
    overflow: auto;
    min-height: 0;
    padding: 0 var(--s2);
  }
  .hint {
    padding: var(--s2) var(--s3);
    font-size: var(--t-sm);
  }
  .list-scroll ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list-scroll li {
    display: flex;
    align-items: center;
    border-radius: var(--r-sm);
  }
  .list-scroll li.active {
    background: var(--accent-soft);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .row-name {
    flex: 1;
    text-align: left;
    background: transparent;
    border: none;
    padding: 0.5rem 0.55rem;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-radius: var(--r-sm);
  }
  .row-name:hover {
    background: transparent;
    color: var(--accent);
  }
  .list-scroll li.active .row-name {
    color: var(--accent);
  }
  .row-del {
    display: grid;
    place-items: center;
    background: transparent;
    border: none;
    color: var(--text-faint);
    padding: 0.3rem 0.45rem;
  }
  .row-del:hover {
    background: transparent;
    color: var(--danger);
  }
  .list-add {
    margin: var(--s2);
    background: transparent;
    border: 1px dashed var(--line-strong);
    color: var(--text-dim);
    font-size: var(--t-sm);
  }
  .list-add:hover {
    color: var(--accent);
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }

  /* ── Detail column ── */
  .detail-col {
    overflow: auto;
    padding: var(--s5);
    display: flex;
    flex-direction: column;
    gap: var(--s4);
    min-height: 0;
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
    font-size: var(--t-xl);
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
  .grow {
    flex: 1;
  }
  button.ghost {
    background: transparent;
    color: var(--text-dim);
    font-size: var(--t-sm);
  }
  .status {
    margin: 0;
    font-size: var(--t-sm);
  }
  .status:empty {
    display: none;
  }

  .section {
    border-top: 1px solid var(--line);
    padding-top: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .section .eyebrow {
    display: block;
  }

  /* Set: song chips */
  .chips {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .chip {
    display: flex;
    align-items: center;
    gap: var(--s3);
    padding: var(--s2) var(--s3);
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
  }
  .chip-idx {
    color: var(--text-faint);
    font-size: var(--t-2xs);
    width: 1.4em;
    text-align: right;
  }
  .chip-name {
    flex: 1;
    font-weight: 500;
  }
  .chip-actions {
    display: flex;
    gap: 2px;
  }

  /* Song: parts + pedals */
  .field-row {
    display: flex;
    gap: var(--s4);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--s1);
  }
  .field input {
    width: 8rem;
  }
  .part {
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: var(--s3);
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .part-head {
    display: flex;
    align-items: center;
    gap: var(--s2);
  }
  .pedals {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s1);
  }
  .pedal {
    display: flex;
    align-items: center;
    gap: var(--s3);
    padding: var(--s2);
    background: var(--inset);
    border-radius: var(--r-sm);
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: var(--s2);
    cursor: pointer;
  }
  .toggle input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .toggle-name {
    font-weight: 500;
  }
  .preset {
    display: flex;
    align-items: center;
    gap: var(--s2);
  }
  .preset .eyebrow {
    display: inline;
  }
  .preset-input {
    min-width: 6rem;
    max-width: 13rem;
    font-family: var(--font-mono);
    font-size: var(--t-sm);
  }
  select.preset-input {
    padding-right: 1.6rem;
  }

  .adder {
    display: flex;
    gap: var(--s2);
  }
  .adder select,
  .adder input {
    flex: 1;
    font-size: var(--t-sm);
  }

  /* Pedal spec read view */
  .spec {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s1);
  }
  .spec li {
    display: flex;
    justify-content: space-between;
    padding: var(--s2) var(--s3);
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
  }
  .spec-key {
    font-weight: 500;
  }

  /* Generic small icon buttons */
  button.icon {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--text-dim);
    padding: 0.15rem 0.45rem;
    border-radius: var(--r-sm);
    font-size: var(--t-sm);
    line-height: 1.2;
  }
  button.icon:hover {
    color: var(--text);
    border-color: var(--line-strong);
  }
  button.icon.danger:hover {
    color: var(--danger);
    border-color: var(--danger);
  }

  /* Advanced */
  .advanced {
    border-top: 1px solid var(--line);
    padding-top: var(--s3);
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .disclose {
    align-self: flex-start;
    background: transparent;
    border: none;
    color: var(--text-dim);
    font-size: var(--t-sm);
    padding: 0;
  }
  .disclose:hover {
    background: transparent;
    color: var(--text);
  }
  .caret {
    display: inline-block;
    transition: transform 0.14s ease;
  }
  .caret.open {
    transform: rotate(90deg);
  }
</style>
