<script lang="ts">
  import { request } from "../lib/transport";
  import { humanizeError } from "../lib/errors";

  let display = "";
  let busy = false;
  let error = "";

  async function dpad(direction: string) {
    await send({ op: "dpad", direction });
  }
  async function short(button: string) {
    await send({ op: "short", button });
  }

  async function send(req: { op: "dpad"; direction: string } | { op: "short"; button: string }) {
    busy = true;
    error = "";
    try {
      const data = await request<{ display_message?: string }>(req);
      display = (data?.display_message ?? "").split(" - ").join("\n");
    } catch (e) {
      error = humanizeError(e);
    } finally {
      busy = false;
    }
  }

  $: lines = display ? display.split("\n") : [];
</script>

<div class="control">
  <div class="surface">
    <!-- Live readout -->
    <div class="readout" role="status" aria-live="polite" aria-label="Controller display">
      <span class="led live"></span>
      <span class="readout-eyebrow eyebrow">Now</span>
      {#if lines.length}
        <div class="readout-text mono">
          {#each lines as line}<span>{line}</span>{/each}
        </div>
      {:else}
        <div class="readout-text mono idle">Ready</div>
      {/if}
    </div>

    {#if error}
      <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
    {/if}

    <!-- Song / Part steppers -->
    <div class="steppers">
      <div class="stepper">
        <span class="label">Song</span>
        <div class="pn">
          <button aria-label="Previous song" on:click={() => short("4")} disabled={busy}>‹</button>
          <button aria-label="Next song" on:click={() => short("5")} disabled={busy}>›</button>
        </div>
      </div>
      <div class="stepper">
        <span class="label">Part</span>
        <div class="pn">
          <button aria-label="Previous part" on:click={() => short("1")} disabled={busy}>‹</button>
          <button aria-label="Next part" on:click={() => short("3")} disabled={busy}>›</button>
        </div>
      </div>
    </div>

    <button class="select primary" on:click={() => short("2")} disabled={busy}>Select</button>

    <!-- Encoder / menu (secondary) -->
    <div class="menu">
      <span class="eyebrow">Menu · encoder</span>
      <div class="menu-keys">
        <button aria-label="Menu up" on:click={() => dpad("up")} disabled={busy}>↑</button>
        <button aria-label="Menu down" on:click={() => dpad("down")} disabled={busy}>↓</button>
        <button aria-label="Rotate counter-clockwise" on:click={() => dpad("CCW")} disabled={busy}>⟲</button>
        <button aria-label="Rotate clockwise" on:click={() => dpad("CW")} disabled={busy}>⟳</button>
      </div>
    </div>
  </div>
</div>

<style>
  .control {
    max-width: 520px;
    margin: 0 auto;
  }
  .surface {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s5);
    display: flex;
    flex-direction: column;
    gap: var(--s4);
  }

  /* Live readout */
  .readout {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s2);
    min-height: 6rem;
    background: var(--inset);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s5) var(--s4);
  }
  .readout .led {
    position: absolute;
    top: var(--s3);
    left: var(--s3);
  }
  .readout-eyebrow {
    position: absolute;
    top: var(--s3);
    left: var(--s5);
  }
  .readout-text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    text-align: center;
    font-size: var(--t-xl);
    letter-spacing: 0.04em;
    color: var(--accent);
    text-shadow: 0 0 12px rgba(245, 165, 36, 0.35);
  }
  .readout-text.idle {
    font-size: var(--t-lg);
    color: var(--text-faint);
    text-shadow: none;
  }

  /* Steppers */
  .steppers {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line);
  }
  .stepper {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s3) var(--s1);
    border-bottom: 1px solid var(--line);
  }
  .stepper .label {
    font-weight: 600;
  }
  .pn {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    background: var(--inset);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
  }
  .pn button {
    border: none;
    background: transparent;
    border-radius: var(--r-sm);
    width: 46px;
    height: 36px;
    font-size: 1.35rem;
    line-height: 1;
    color: var(--text-dim);
  }
  .pn button:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  /* Select — the one prominent action */
  .select {
    padding: var(--s4);
    font-size: var(--t-base);
  }

  /* Encoder / menu — secondary */
  .menu {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .menu-keys {
    display: flex;
    gap: var(--s2);
  }
  .menu-keys button {
    flex: 1;
    height: 42px;
    font-size: 1.1rem;
    color: var(--text-dim);
  }
  .menu-keys button:hover {
    color: var(--text);
  }
</style>
