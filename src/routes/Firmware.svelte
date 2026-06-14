<script lang="ts">
  import { connection } from "../lib/stores";
  import { request, disconnectDevice } from "../lib/transport";
  import { humanizeError } from "../lib/errors";

  let busy = false;
  let error = "";

  $: identity = $connection.identity;

  // reboot / reboot_bootloader: the firmware acks, then drops the link ~100ms
  // later — both the ack and the drop mean success. We then disconnect locally
  // so the app returns to the Connect screen (where the bootloader appears, for
  // reboot_bootloader, to finish flashing).
  async function fire(run: () => Promise<unknown>) {
    busy = true;
    error = "";
    try {
      await run().catch(() => {});
      await disconnectDevice().catch(() => {});
    } catch (e) {
      error = humanizeError(e);
    } finally {
      busy = false;
    }
  }

  const reboot = () => fire(() => request({ op: "reboot" }));
  const updateFirmware = () => fire(() => request({ op: "reboot_bootloader" }));
</script>

<div class="firmware">
  <div class="surface">
    <header class="head">
      <h2>Firmware</h2>
      <span class="grow"></span>
      {#if identity}<span class="ver mono">v{identity.firmware}</span>{/if}
    </header>

    {#if identity?.device_id}
      <p class="muted hint">Device ID <span class="mono">{identity.device_id}</span></p>
    {/if}

    {#if error}
      <div class="notice err" role="alert"><span class="ic">⚠</span><span>{error}</span></div>
    {/if}

    <div class="row">
      <div class="row-text">
        <strong>Reboot</strong>
        <span class="muted">Soft restart into the app.</span>
      </div>
      <button on:click={reboot} disabled={busy}>Reboot</button>
    </div>

    <div class="row">
      <div class="row-text">
        <strong>Update firmware</strong>
        <span class="muted">Restarts into the USB bootloader — finish flashing from the Connect screen.</span>
      </div>
      <button class="primary" on:click={updateFirmware} disabled={busy}>Update firmware…</button>
    </div>

    <p class="muted hint cable">
      ⓘ Flashing always needs a USB cable — even when connected over Wi-Fi, the
      device lands in USB bootloader mode.
    </p>
  </div>
</div>

<style>
  .firmware {
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
    gap: var(--s3);
  }
  .head {
    display: flex;
    align-items: center;
    gap: var(--s2);
  }
  .head h2 {
    font-size: var(--t-lg);
  }
  .grow {
    flex: 1;
  }
  .ver {
    font-size: var(--t-sm);
    color: var(--accent);
  }
  .hint {
    margin: 0;
    font-size: var(--t-sm);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--s4);
    padding: var(--s3) 0;
    border-top: 1px solid var(--line);
  }
  .row-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  .row-text .muted {
    font-size: var(--t-sm);
  }
  .cable {
    margin-top: var(--s1);
    color: var(--text-faint);
  }
  .notice .ic {
    font-weight: 700;
  }
</style>
