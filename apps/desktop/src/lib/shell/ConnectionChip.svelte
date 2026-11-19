<script lang="ts">
  import type { AppState } from "../state/app.svelte";
  import ToneIcon from "../ToneIcon.svelte";
  import Popover from "./Popover.svelte";
  import { connectionName, type ConnectionStatus } from "./status";

  let {
    app,
    status,
    background,
    onconnect,
  }: { app: AppState; status: ConnectionStatus; background: string | null; onconnect: () => void } = $props();

  let busy = $derived(app.working !== null);
</script>

<Popover id="connection-panel" label={connectionName(status)} panelLabel="Connection details">
  {#snippet trigger()}
    <ToneIcon tone={status.tone} />
    <span class="label">{status.label}</span>
    {#if status.where}<span class="where">{status.where}</span>{/if}
  {/snippet}
  {#snippet panel(close)}
    <div class="lines">
      {#each status.lines as line (line)}
        <p>{line}</p>
      {/each}
      {#if background}<p class="quiet">{background}</p>{/if}
    </div>
    <div class="actions">
      {#if status.step === "live"}
        <button type="button" disabled={busy} onclick={() => (close(), app.refresh())}>Rescan devices</button>
        <button type="button" disabled={app.working === "disconnect"} onclick={() => (close(), app.disconnect())}>Disconnect</button>
      {:else if status.step === "retrying"}
        <button type="button" disabled={app.working === "disconnect"} onclick={() => (close(), app.disconnect())}>Stop retrying</button>
      {:else if status.step === "connecting" || status.step === "starting"}
        <button type="button" disabled={app.working === "disconnect"} onclick={() => (close(), app.disconnect())}>Disconnect</button>
      {:else}
        <button type="button" class="primary" onclick={() => (close(), onconnect())}>Connect to OpenRGB</button>
        <button type="button" disabled={busy} onclick={() => (close(), app.startDemo())}>Start demo mode</button>
      {/if}
    </div>
  {/snippet}
</Popover>

<style>
  .label {
    font-weight: var(--weight-strong);
    white-space: nowrap;
  }

  .where {
    color: var(--text-dim);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lines {
    display: grid;
    gap: var(--space-1);
  }

  .quiet {
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
