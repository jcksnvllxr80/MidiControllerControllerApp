<script lang="ts">
  import { sidebarCollapsed } from "./lib/stores";

  // The active view + a setter, owned by App. `onDisconnect` ends the session.
  export let view: string;
  export let onSelect: (v: string) => void;
  export let onDisconnect: () => void;

  const ITEMS: { key: string; label: string }[] = [
    { key: "control", label: "Control" },
    { key: "configure", label: "Configure" },
    { key: "json", label: "JSON" },
    { key: "wifi", label: "Wi-Fi" },
    { key: "firmware", label: "Firmware" },
  ];

  const toggle = () => sidebarCollapsed.update((v) => !v);
</script>

<aside class="sidebar" class:collapsed={$sidebarCollapsed}>
  <nav class="items" aria-label="Views">
    {#each ITEMS as it (it.key)}
      <button
        class="item"
        class:active={view === it.key}
        aria-current={view === it.key ? "page" : undefined}
        aria-label={it.label}
        title={$sidebarCollapsed ? it.label : undefined}
        on:click={() => onSelect(it.key)}
      >
        <span class="ic" aria-hidden="true">
          {#if it.key === "control"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="8" /><line x1="12" y1="12" x2="12" y2="5.5" /></svg>
          {:else if it.key === "configure"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="8" x2="20" y2="8" /><circle cx="9" cy="8" r="2.3" fill="var(--panel)" /><line x1="4" y1="16" x2="20" y2="16" /><circle cx="15" cy="16" r="2.3" fill="var(--panel)" /></svg>
          {:else if it.key === "json"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M8 4c-1.6 0-2.4.9-2.4 2.4v2.2c0 1-.5 1.4-1.6 1.4 1.1 0 1.6.4 1.6 1.4v2.2C5.6 17.1 6.4 18 8 18" /><path d="M16 4c1.6 0 2.4.9 2.4 2.4v2.2c0 1 .5 1.4 1.6 1.4-1.1 0-1.6.4-1.6 1.4v2.2C18.4 17.1 17.6 18 16 18" /></svg>
          {:else if it.key === "wifi"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9a13 13 0 0 1 16 0" /><path d="M7 12.5a8 8 0 0 1 10 0" /><path d="M9.7 15.7a4 4 0 0 1 4.6 0" /><circle cx="12" cy="18.6" r="0.7" fill="currentColor" /></svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="7" width="10" height="10" rx="1.5" /><rect x="10" y="10" width="4" height="4" rx="0.5" /><line x1="9.5" y1="4" x2="9.5" y2="7" /><line x1="14.5" y1="4" x2="14.5" y2="7" /><line x1="9.5" y1="17" x2="9.5" y2="20" /><line x1="14.5" y1="17" x2="14.5" y2="20" /><line x1="4" y1="9.5" x2="7" y2="9.5" /><line x1="4" y1="14.5" x2="7" y2="14.5" /><line x1="17" y1="9.5" x2="20" y2="9.5" /><line x1="17" y1="14.5" x2="20" y2="14.5" /></svg>
          {/if}
        </span>
        {#if !$sidebarCollapsed}<span class="label">{it.label}</span>{/if}
      </button>
    {/each}
  </nav>

  <div class="footer">
    <button
      class="item disconnect"
      aria-label="Disconnect"
      title={$sidebarCollapsed ? "Disconnect" : undefined}
      on:click={onDisconnect}
    >
      <span class="ic" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4v8" /><path d="M7.6 7.6a7 7 0 1 0 8.8 0" /></svg>
      </span>
      {#if !$sidebarCollapsed}<span class="label">Disconnect</span>{/if}
    </button>
  </div>

  <button
    class="toggle"
    class:collapsed={$sidebarCollapsed}
    aria-label={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
    on:click={toggle}
  >
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 7l-5 5 5 5" /></svg>
  </button>
</aside>

<style>
  .sidebar {
    position: relative;
    flex: none;
    width: 212px;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-right: 1px solid var(--line);
    transition: width 0.16s ease;
  }
  .sidebar.collapsed {
    width: 56px;
  }

  .items {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--s3) var(--s2);
  }
  .item {
    display: flex;
    align-items: center;
    gap: var(--s3);
    width: 100%;
    padding: 0.55rem 0.6rem;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-md);
    color: var(--text-dim);
    overflow: hidden;
  }
  .item:hover {
    background: var(--panel-2);
    color: var(--text);
    border-color: transparent;
  }
  .item.active {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .sidebar.collapsed .item {
    justify-content: center;
    padding: 0.55rem 0;
  }
  .ic {
    flex: none;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
  }
  .label {
    white-space: nowrap;
    font-size: var(--t-sm);
    font-weight: 500;
  }

  .footer {
    margin-top: auto;
    padding: var(--s2);
    border-top: 1px solid var(--line);
  }
  .disconnect:hover {
    color: var(--danger);
    background: var(--panel-2);
  }

  /* Collapse/expand handle, straddling the right border line. */
  .toggle {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    right: -12px;
    width: 24px;
    height: 24px;
    padding: 0;
    display: grid;
    place-items: center;
    border-radius: 50%;
    background: var(--control);
    border: 1px solid var(--line);
    color: var(--text-dim);
    z-index: 3;
  }
  .toggle:hover {
    color: var(--text);
    border-color: var(--line-strong);
  }
  .toggle svg {
    transition: transform 0.16s ease;
  }
  .toggle.collapsed svg {
    transform: rotate(180deg);
  }
</style>
