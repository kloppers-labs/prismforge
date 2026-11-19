<script lang="ts">
  import type { AppState } from "../state/app.svelte";
  import ToneIcon from "../ToneIcon.svelte";
  import Popover from "./Popover.svelte";
  import { connectionName, type ConnectionStatus } from "./status";
  import { busyText, connectionMenu } from "./stripMenus";

  let {
    app,
    status,
    background,
    onconnect,
  }: { app: AppState; status: ConnectionStatus; background: string | null; onconnect: () => void } = $props();

  let busy = $derived(app.working !== null);
  let endsOnly = $derived(status.step !== "live" && status.step !== "none" && status.step !== "ended");

  function act(close: () => void, action: () => unknown) {
    close();
    app.uncover();
    void action();
  }
</script>

<Popover
  id="connection-panel"
  label={connectionName(status)}
  panelLabel="Connection details"
  focus={endsOnly ? "panel" : "first"}
  menu={() => connectionMenu(app, status, onconnect)}
>
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
    {#if status.step === "live"}
      <div class="actions">
        <button
          type="button"
          data-mirror="connection.rescan"
          aria-disabled={busy ? "true" : undefined}
          aria-describedby={busy ? "rescan-reason" : undefined}
          onclick={() => busy || act(close, () => app.refresh())}
        >
          Rescan devices
        </button>
      </div>
      {#if busy}<p id="rescan-reason" class="quiet">{busyText(app)}</p>{/if}
    {:else if status.step === "none" || status.step === "ended"}
      <div class="actions">
        <button type="button" class="primary" data-mirror="connection.connect" onclick={() => act(close, onconnect)}>Connect to OpenRGB</button>
        <button type="button" data-mirror="connection.demo" disabled={busy} onclick={() => act(close, () => app.startDemo())}>Start demo mode</button>
      </div>
    {/if}
    {#if status.step !== "none" && status.step !== "ended"}
      <div class="end">
        <button type="button" class="danger" data-mirror="connection.end" disabled={app.working === "disconnect"} onclick={() => act(close, () => app.disconnect())}>
          {status.step === "retrying" ? "Stop retrying" : "Disconnect"}
        </button>
        <p class="quiet">Ends the connection and stops every PrismForge effect.</p>
      </div>
    {/if}
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

  .end {
    display: grid;
    justify-items: start;
    gap: var(--space-1);
    padding-top: var(--space-3);
    border-top: 1px solid var(--line-subtle);
  }

  [aria-disabled="true"] {
    cursor: not-allowed;
    color: var(--text-faint);
    background: transparent;
    border-color: var(--line-subtle);
  }
</style>
