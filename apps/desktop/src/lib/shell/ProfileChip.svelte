<script lang="ts">
  import type { AppState } from "../state/app.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import type { ProfileStatus } from "./status";

  let { app, status }: { app: AppState; status: ProfileStatus } = $props();
</script>

<Popover id="profile-panel" label={status.name} panelLabel="Profile details">
  {#snippet trigger()}
    <span class="icon" class:busy={status.busy}><Icon name="profiles" /></span>
    <span class="title">{status.title}</span>
    {#if status.detail}<span class="detail">{status.detail}</span>{/if}
  {/snippet}
  {#snippet panel(close)}
    <div class="lines">
      {#each status.lines as line (line)}
        <p>{line}</p>
      {/each}
    </div>
    <div class="actions">
      {#if status.saveable}
        {@const id = status.saveable}
        <button type="button" onclick={() => (close(), app.uncover(), app.saveActiveChanges(id))}>Save changes…</button>
      {/if}
      <button type="button" onclick={() => (close(), app.uncover(), app.openProfiles())}>Open Profiles</button>
    </div>
  {/snippet}
</Popover>

<style>
  .icon {
    display: inline-flex;
    color: var(--text-dim);
  }

  .icon.busy {
    color: var(--working);
    animation: pulse 1.2s var(--ease-out) infinite alternate;
  }

  @keyframes pulse {
    to {
      opacity: 0.45;
    }
  }

  .title {
    font-weight: var(--weight-strong);
    white-space: nowrap;
    min-width: 0;
    max-width: 22ch;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detail {
    font-size: var(--font-xs);
    white-space: nowrap;
    padding: 0 var(--space-2);
    border: 1px solid var(--warning-line);
    border-radius: var(--radius-xs);
    color: var(--text);
  }

  .lines {
    display: grid;
    gap: var(--space-1);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
