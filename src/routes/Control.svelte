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
  <textarea class="display" readonly rows="4" value={display} placeholder="Controller display…"></textarea>

  {#if error}<p class="error">{error}</p>{/if}

  <div class="dpad">
    <button class="up" on:click={() => dpad("up")} disabled={busy}>↑</button>
    <button class="left" on:click={() => dpad("CCW")} disabled={busy}>←</button>
    <span class="center"></span>
    <button class="right" on:click={() => dpad("CW")} disabled={busy}>→</button>
    <button class="down" on:click={() => dpad("down")} disabled={busy}>↓</button>
  </div>

  <div class="buttons">
    <div class="row">
      <button on:click={() => short("4")} disabled={busy}>Song ↓</button>
      <button on:click={() => short("5")} disabled={busy}>Song ↑</button>
    </div>
    <div class="row">
      <button on:click={() => short("1")} disabled={busy}>Part ↓</button>
      <button class="select" on:click={() => short("2")} disabled={busy}>Select</button>
      <button on:click={() => short("3")} disabled={busy}>Part ↑</button>
    </div>
  </div>
</div>

<style>
  .control {
    max-width: 460px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }
  .display {
    text-align: center;
    font-size: 1.05rem;
    background: #0a0c12;
    border-color: var(--accent);
    color: var(--good);
    min-height: 5.5rem;
  }
  .dpad {
    display: grid;
    grid-template-columns: repeat(3, 64px);
    grid-template-rows: repeat(3, 64px);
    justify-content: center;
    gap: 0.5rem;
  }
  .dpad button {
    font-size: 1.4rem;
  }
  .dpad .up {
    grid-column: 2;
    grid-row: 1;
  }
  .dpad .left {
    grid-column: 1;
    grid-row: 2;
  }
  .dpad .center {
    grid-column: 2;
    grid-row: 2;
  }
  .dpad .right {
    grid-column: 3;
    grid-row: 2;
  }
  .dpad .down {
    grid-column: 2;
    grid-row: 3;
  }
  .buttons {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .row {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
  }
  .row button {
    flex: 1;
    padding: 0.9rem;
  }
  .select {
    border-color: var(--accent);
  }
</style>
