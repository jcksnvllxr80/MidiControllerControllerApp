<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { connection } from "../lib/stores";
  import { request } from "../lib/transport";
  import { humanizeError } from "../lib/errors";

  let display = "";
  let busy = false;
  let error = "";

  // Long-press thresholds
  const LONG_MS = 600;
  const EXTRA_LONG_MS = 1500;

  // Which numbered button (1..5) is currently being held, and how far along
  let holdingBtn: string | null = null;
  let holdLevel = 0; // 0 = held but not long yet, 1 = long, 2 = extra-long
  let holdStart = 0;
  let longTimer: ReturnType<typeof setTimeout> | undefined;
  let extraLongTimer: ReturnType<typeof setTimeout> | undefined;

  function pointerDown(e: PointerEvent, button: string) {
    if (busy) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    holdingBtn = button;
    holdLevel = 0;
    holdStart = Date.now();
    longTimer = setTimeout(() => { holdLevel = 1; }, LONG_MS);
    extraLongTimer = setTimeout(() => { holdLevel = 2; }, EXTRA_LONG_MS);
  }

  function pointerUp(button: string) {
    clearHoldTimers();
    if (!holdingBtn) return;
    const elapsed = Date.now() - holdStart;
    const lvl = holdLevel;
    holdingBtn = null;
    holdLevel = 0;
    if (elapsed >= EXTRA_LONG_MS || lvl >= 2) {
      send({ op: "extra_long", button });
    } else if (elapsed >= LONG_MS || lvl >= 1) {
      send({ op: "long", button });
    } else {
      send({ op: "short", button });
    }
  }

  function pointerCancel() {
    clearHoldTimers();
    holdingBtn = null;
    holdLevel = 0;
  }

  function clearHoldTimers() {
    clearTimeout(longTimer);
    clearTimeout(extraLongTimer);
    longTimer = undefined;
    extraLongTimer = undefined;
  }

  async function dpad(direction: string) {
    await send({ op: "dpad", direction });
  }

  type ControlReq =
    | { op: "dpad"; direction: string }
    | { op: "short"; button: string }
    | { op: "long"; button: string }
    | { op: "extra_long"; button: string };

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
      if (!$connection.connected || busy || polling || holdingBtn) return;
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
    clearHoldTimers();
  });

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
          <button
            aria-label="Previous song"
            class:holding={holdingBtn === "4"}
            class:long-held={holdingBtn === "4" && holdLevel >= 1}
            class:extra-long-held={holdingBtn === "4" && holdLevel >= 2}
            on:pointerdown={(e) => pointerDown(e, "4")}
            on:pointerup={() => pointerUp("4")}
            on:pointercancel={pointerCancel}
            disabled={busy}>‹</button>
          <button
            aria-label="Next song"
            class:holding={holdingBtn === "5"}
            class:long-held={holdingBtn === "5" && holdLevel >= 1}
            class:extra-long-held={holdingBtn === "5" && holdLevel >= 2}
            on:pointerdown={(e) => pointerDown(e, "5")}
            on:pointerup={() => pointerUp("5")}
            on:pointercancel={pointerCancel}
            disabled={busy}>›</button>
        </div>
      </div>
      <div class="stepper">
        <span class="label">Part</span>
        <div class="pn">
          <button
            aria-label="Previous part"
            class:holding={holdingBtn === "1"}
            class:long-held={holdingBtn === "1" && holdLevel >= 1}
            class:extra-long-held={holdingBtn === "1" && holdLevel >= 2}
            on:pointerdown={(e) => pointerDown(e, "1")}
            on:pointerup={() => pointerUp("1")}
            on:pointercancel={pointerCancel}
            disabled={busy}>‹</button>
          <button
            aria-label="Next part"
            class:holding={holdingBtn === "3"}
            class:long-held={holdingBtn === "3" && holdLevel >= 1}
            class:extra-long-held={holdingBtn === "3" && holdLevel >= 2}
            on:pointerdown={(e) => pointerDown(e, "3")}
            on:pointerup={() => pointerUp("3")}
            on:pointercancel={pointerCancel}
            disabled={busy}>›</button>
        </div>
      </div>
    </div>

    <button
      class="select primary"
      class:holding={holdingBtn === "2"}
      class:long-held={holdingBtn === "2" && holdLevel >= 1}
      class:extra-long-held={holdingBtn === "2" && holdLevel >= 2}
      on:pointerdown={(e) => pointerDown(e, "2")}
      on:pointerup={() => pointerUp("2")}
      on:pointercancel={pointerCancel}
      disabled={busy}>Select</button>

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
    /* Fixed display height — never grows/shrinks with the message (1–4 lines all
       stay vertically centered in the same box). */
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
    transition: background 0.15s, color 0.15s;
    user-select: none;
    touch-action: none;
  }
  .pn button:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  /* Hold-state feedback for numbered buttons */
  .pn button.holding,
  .select.holding {
    background: var(--panel-2);
    color: var(--text);
  }
  .pn button.long-held,
  .select.long-held {
    background: color-mix(in srgb, var(--accent) 20%, var(--panel-2));
    color: var(--accent);
  }
  .pn button.extra-long-held,
  .select.extra-long-held {
    background: color-mix(in srgb, var(--accent) 40%, var(--panel-2));
    color: var(--accent);
  }

  /* Select — the one prominent action */
  .select {
    padding: var(--s4);
    font-size: var(--t-base);
    user-select: none;
    touch-action: none;
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
