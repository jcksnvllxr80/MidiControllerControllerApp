<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { connection } from "../lib/stores";
  import { request } from "../lib/transport";
  import { humanizeError } from "../lib/errors";

  let display = "";
  let busy = false;
  let error = "";

  // Encoder push hold thresholds (ms) — mirror firmware's MenuTree::pressFor
  const ENC_LONG_MS = 1000;       // dpad("up")   — sub-menu / back
  const ENC_GLOBAL_MS = 3000;     // long          — global menu
  const ENC_POWER_MS = 6000;      // extra_long    — power menu

  // Hold state for the encoder push button
  let encHolding = false;
  let encLevel = 0;  // 0 = holding, 1 = long, 2 = global, 3 = power
  let encStart = 0;
  let encTimer1: ReturnType<typeof setTimeout> | undefined;
  let encTimer2: ReturnType<typeof setTimeout> | undefined;
  let encTimer3: ReturnType<typeof setTimeout> | undefined;

  function encPointerDown(e: PointerEvent) {
    if (busy) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    encHolding = true;
    encLevel = 0;
    encStart = Date.now();
    encTimer1 = setTimeout(() => { encLevel = 1; }, ENC_LONG_MS);
    encTimer2 = setTimeout(() => { encLevel = 2; }, ENC_GLOBAL_MS);
    encTimer3 = setTimeout(() => { encLevel = 3; }, ENC_POWER_MS);
  }

  function encPointerUp() {
    clearEncTimers();
    if (!encHolding) return;
    const elapsed = Date.now() - encStart;
    const lvl = encLevel;
    encHolding = false;
    encLevel = 0;
    if (elapsed >= ENC_POWER_MS || lvl >= 3) {
      send({ op: "extra_long" });
    } else if (elapsed >= ENC_GLOBAL_MS || lvl >= 2) {
      send({ op: "long" });
    } else if (elapsed >= ENC_LONG_MS || lvl >= 1) {
      send({ op: "dpad", direction: "up" });
    } else {
      send({ op: "dpad", direction: "down" });
    }
  }

  function encPointerCancel() {
    clearEncTimers();
    encHolding = false;
    encLevel = 0;
  }

  function clearEncTimers() {
    clearTimeout(encTimer1);
    clearTimeout(encTimer2);
    clearTimeout(encTimer3);
    encTimer1 = encTimer2 = encTimer3 = undefined;
  }

  async function short(button: string) {
    await send({ op: "short", button });
  }

  async function dpad(direction: string) {
    await send({ op: "dpad", direction });
  }

  type ControlReq =
    | { op: "dpad"; direction: string }
    | { op: "short"; button: string }
    | { op: "long" }
    | { op: "extra_long" };

  async function send(req: ControlReq) {
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

  // Background display poll — keeps the readout in sync when hardware buttons
  // are pressed directly on the device.
  let polling = false;
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  onMount(() => {
    pollTimer = setInterval(async () => {
      if (!$connection.connected || busy || polling || encHolding) return;
      polling = true;
      try {
        const data = await request<{ display_message?: string }>({ op: "get_display" });
        if (data?.display_message != null) {
          display = data.display_message.split(" - ").join("\n");
        }
      } catch {
        // silent — poll failures don't surface as errors
      } finally {
        polling = false;
      }
    }, 500);
  });

  onDestroy(() => {
    clearInterval(pollTimer);
    clearEncTimers();
  });

  $: lines = display ? display.split("\n") : [];

  // Label shown on the encoder push button during a hold
  $: encHoldLabel =
    encLevel >= 3 ? "Power menu" :
    encLevel >= 2 ? "Global menu" :
    encLevel >= 1 ? "Back / sub-menu" :
    "Push";
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

    <!-- Encoder section -->
    <div class="menu">
      <span class="eyebrow">Encoder</span>
      <div class="enc-row">
        <div class="enc-rotate">
          <button aria-label="Rotate counter-clockwise" on:click={() => dpad("CCW")} disabled={busy}>⟲</button>
          <button aria-label="Rotate clockwise" on:click={() => dpad("CW")} disabled={busy}>⟳</button>
        </div>
        <!-- Encoder push button — tap to confirm, hold for menus -->
        <button
          class="enc-push"
          class:holding={encHolding && encLevel === 0}
          class:level-1={encLevel >= 1}
          class:level-2={encLevel >= 2}
          class:level-3={encLevel >= 3}
          aria-label="Encoder push — tap to confirm, hold for menus"
          on:pointerdown={encPointerDown}
          on:pointerup={encPointerUp}
          on:pointercancel={encPointerCancel}
          disabled={busy}
        >{encHolding ? encHoldLabel : "Push"}</button>
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
    height: 182px;
    overflow: hidden;
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

  /* Select */
  .select {
    padding: var(--s4);
    font-size: var(--t-base);
  }

  /* Encoder section */
  .menu {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .enc-row {
    display: flex;
    gap: var(--s2);
    align-items: stretch;
  }
  .enc-rotate {
    display: flex;
    gap: var(--s2);
  }
  .enc-rotate button {
    flex: 1;
    height: 42px;
    width: 52px;
    font-size: 1.1rem;
    color: var(--text-dim);
  }
  .enc-rotate button:hover {
    color: var(--text);
  }

  /* Encoder push — the hold target */
  .enc-push {
    flex: 1;
    height: 42px;
    font-size: var(--t-sm);
    font-weight: 600;
    color: var(--text-dim);
    transition: background 0.2s, color 0.2s;
    user-select: none;
    touch-action: none;
  }
  .enc-push:hover {
    color: var(--text);
  }
  .enc-push.holding {
    background: var(--panel-2);
    color: var(--text);
  }
  /* 1 s — sub-menu / back */
  .enc-push.level-1 {
    background: color-mix(in srgb, var(--accent) 15%, var(--panel-2));
    color: var(--accent);
  }
  /* 3 s — global menu */
  .enc-push.level-2 {
    background: color-mix(in srgb, var(--accent) 30%, var(--panel-2));
    color: var(--accent);
  }
  /* 6 s — power menu */
  .enc-push.level-3 {
    background: color-mix(in srgb, var(--accent) 50%, var(--panel-2));
    color: var(--accent);
  }
</style>
