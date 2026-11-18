<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AppState } from "../../lib/state/app.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import { isLive } from "../devices/present";
  import { place, summarise } from "./wording";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let summary = $derived(summarise(session));
</script>

<div class="bar">
  <p class="state" aria-live="polite">
    <ToneIcon tone={summary.tone} />
    <span class="label">{summary.label}</span>
    <span class="where">{place(session)}</span>
  </p>
  <div class="actions">
    {#if isLive(session)}
      <button type="button" onclick={() => app.refresh()} disabled={app.working !== null}>
        {app.working === "refresh" ? "Refreshing" : "Refresh devices"}
      </button>
    {/if}
    <button type="button" onclick={() => app.disconnect()} disabled={app.working === "disconnect"}>
      {summary.retrying ? "Stop retrying" : "Disconnect"}
    </button>
  </div>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
    flex: 1;
  }

  .state {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .label {
    font-weight: 600;
    white-space: nowrap;
  }

  .where {
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
</style>
