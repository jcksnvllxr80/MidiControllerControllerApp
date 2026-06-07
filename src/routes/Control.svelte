<script lang="ts">
  import { request } from "../lib/transport";

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
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="control">
  <div class="panel screen">
    <span class="eyebrow">Display</span>
    <textarea class="display mono" readonly rows="3" value={display} placeholder="—"></textarea>
    {#if error}<p class="error">{error}</p>{/if}
  </div>

  <div class="panel">
    <span class="eyebrow">Navigate</span>
    <div class="dpad">
      <button class="key up" on:click={() => dpad("up")} disabled={busy}>↑</button>
      <button class="key left" on:click={() => dpad("CCW")} disabled={busy}>←</button>
      <span class="hub"></span>
      <button class="key right" on:click={() => dpad("CW")} disabled={busy}>→</button>
      <button class="key down" on:click={() => dpad("down")} disabled={busy}>↓</button>
    </div>
  </div>

  <div class="panel">
    <span class="eyebrow">Footswitches</span>
    <div class="switches">
      <button class="fsw" on:click={() => short("4")} disabled={busy}>Song ↓</button>
      <button class="fsw" on:click={() => short("5")} disabled={busy}>Song ↑</button>
      <button class="fsw" on:click={() => short("1")} disabled={busy}>Part ↓</button>
      <button class="fsw select" on:click={() => short("2")} disabled={busy}>Select</button>
      <button class="fsw" on:click={() => short("3")} disabled={busy}>Part ↑</button>
    </div>
  </div>
</div>

<style>
  .control {
    max-width: 460px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--s4);
  }
  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .panel .eyebrow {
    display: block;
  }

  /* LCD readout */
  .screen .display {
    text-align: center;
    font-size: var(--t-lg);
    letter-spacing: 0.02em;
    background: var(--inset);
    border-color: var(--line);
    color: var(--accent);
    min-height: 4.75rem;
    padding: var(--s3);
    text-shadow: 0 0 12px rgba(245, 165, 36, 0.35);
  }
  .screen .display::placeholder {
    color: var(--text-faint);
    text-shadow: none;
  }

  /* D-pad */
  .dpad {
    display: grid;
    grid-template-columns: repeat(3, 62px);
    grid-template-rows: repeat(3, 62px);
    justify-content: center;
    gap: var(--s2);
  }
  .key {
    font-size: 1.3rem;
    border-radius: var(--r-md);
    background: var(--control);
  }
  .hub {
    grid-column: 2;
    grid-row: 2;
    border-radius: 50%;
    background: var(--inset);
    border: 1px solid var(--line);
    margin: 12px;
  }
  .key.up {
    grid-column: 2;
    grid-row: 1;
  }
  .key.left {
    grid-column: 1;
    grid-row: 2;
  }
  .key.right {
    grid-column: 3;
    grid-row: 2;
  }
  .key.down {
    grid-column: 2;
    grid-row: 3;
  }

  /* Footswitches */
  .switches {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--s2);
  }
  .fsw {
    padding: var(--s4) var(--s2);
    border-radius: var(--r-md);
    background: linear-gradient(var(--control), var(--panel-2));
    border-top: 1px solid var(--line-strong);
    font-weight: 500;
  }
  .fsw:active {
    background: var(--panel-2);
  }
  .fsw.select {
    grid-column: 1 / -1;
    color: var(--accent);
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }
  .fsw.select:hover {
    background: var(--accent);
    color: var(--accent-ink);
    border-color: transparent;
  }
</style>
