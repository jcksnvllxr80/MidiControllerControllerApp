<script lang="ts">
  import { theme, THEMES, type ThemeId } from "../lib/theme";

  const groups: { title: string; group: "dark" | "light" | "system" }[] = [
    { title: "Dark", group: "dark" },
    { title: "Light", group: "light" },
    { title: "System", group: "system" },
  ];

  const select = (id: ThemeId) => theme.set(id);
</script>

<section class="appearance">
  <header>
    <p class="eyebrow">Appearance</p>
    <h2>Theme</h2>
    <p class="muted sub">Pick a palette. Every view remaps to it instantly, and your choice is remembered.</p>
  </header>

  {#each groups as g (g.group)}
    {@const items = THEMES.filter((t) => t.group === g.group)}
    <p class="eyebrow group-label">{g.title}</p>
    <div class="grid">
      {#each items as t (t.id)}
        <button
          class="card"
          class:active={$theme === t.id}
          aria-pressed={$theme === t.id}
          on:click={() => select(t.id)}
        >
          {#if t.id === "match-os"}
            <span class="swatch os" aria-hidden="true">
              <span class="os-dark"></span><span class="os-light"></span>
            </span>
          {:else}
            <span class="swatch" data-theme={t.id === "studio" ? undefined : t.id} aria-hidden="true">
              <span class="bar"></span>
              <span class="dot"></span>
              <span class="line a"></span>
              <span class="line b"></span>
            </span>
          {/if}
          <span class="meta">
            <span class="name">{t.label}</span>
            {#if $theme === t.id}<span class="check" aria-hidden="true">✓</span>{/if}
          </span>
        </button>
      {/each}
    </div>
  {/each}
</section>

<style>
  .appearance {
    max-width: 760px;
  }
  header {
    margin-bottom: var(--s5);
  }
  h2 {
    font-size: var(--t-2xl);
    margin-top: var(--s1);
  }
  .sub {
    margin: var(--s2) 0 0;
    font-size: var(--t-sm);
  }
  .group-label {
    margin: var(--s5) 0 var(--s2);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: var(--s3);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
    align-items: stretch;
    padding: var(--s3);
    text-align: left;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
  }
  .card:hover {
    border-color: var(--line-strong);
    background: var(--panel);
  }
  .card.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent), var(--accent-soft) 0 0 0 4px;
  }

  /* A mini render of the theme using its own tokens (self-themed via data-theme). */
  .swatch {
    position: relative;
    height: 64px;
    border-radius: var(--r-md);
    background: var(--bg);
    border: 1px solid var(--line);
    overflow: hidden;
    padding: 10px;
  }
  .swatch .bar {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 22px;
    background: var(--panel-2);
    border-right: 1px solid var(--line);
  }
  .swatch .dot {
    position: absolute;
    top: 10px;
    left: 32px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .swatch .line {
    position: absolute;
    left: 32px;
    height: 6px;
    border-radius: 3px;
    background: var(--text-dim);
  }
  .swatch .line.a {
    top: 30px;
    width: 96px;
    opacity: 0.85;
  }
  .swatch .line.b {
    top: 44px;
    width: 62px;
    opacity: 0.5;
  }

  /* Match-OS: split dark/light preview. */
  .swatch.os {
    display: flex;
    padding: 0;
  }
  .os-dark,
  .os-light {
    flex: 1;
  }
  .os-dark {
    background: #16161a;
  }
  .os-light {
    background: #eee8d5;
  }

  .meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s2);
  }
  .name {
    font-size: var(--t-sm);
    font-weight: 500;
    color: var(--text);
  }
  .check {
    color: var(--accent);
    font-weight: 700;
  }
</style>
