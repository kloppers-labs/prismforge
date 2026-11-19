<script lang="ts">
  import { untrack } from "svelte";
  import { failureText } from "../../lib/errors";
  import type { AppState } from "../../lib/state/app.svelte";
  import { isLive } from "../devices/present";
  import { applyingText } from "../effects/present";
  import ImportResult from "./ImportResult.svelte";
  import type { Labels } from "./present";
  import ProfileArticle from "./ProfileArticle.svelte";
  import ProfileList from "./ProfileList.svelte";
  import RecoveryNotices from "./RecoveryNotices.svelte";
  import ResultPanel from "./ResultPanel.svelte";
  import SaveCurrent from "./SaveCurrent.svelte";

  let { app }: { app: AppState } = $props();

  let profiles = $derived(app.profiles);
  let store = $derived(profiles.store);
  let session = $derived(app.session);
  let live = $derived(session !== null && isLive(session));
  let applying = $derived(applyingText(app.view));
  let details = $derived(profiles.details && profiles.details.id === profiles.selected ? profiles.details : null);
  let labels: Labels = $derived({
    names: profiles.names,
    session,
    effect: (id: string) => app.effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? null,
  });
  let nameField: HTMLInputElement | undefined = $state();

  let connected = $derived(session !== null);

  $effect(() => {
    if (connected) {
      untrack(() => app.effects.load());
    }
  });
</script>

<div class="screen">
  <ProfileList {profiles} {store} />
  <div class="workspace">
    {#if profiles.storeProblem}
      <div class="notice" role="alert">
        <p>{failureText(profiles.storeProblem)}</p>
        <button type="button" onclick={() => profiles.refresh()}>Try again</button>
      </div>
    {:else if !store}
      <p class="quiet" role="status">Reading the profiles.</p>
    {:else}
      {#if store.kind === "Demo"}
        <p class="mode">These are demo profiles. They are kept apart from your real profiles and never apply to real hardware.</p>
      {/if}
      {#if !live}
        <div class="mode">
          <p>
            PrismForge is not connected. You can still rename, duplicate, delete, export and import profiles, and choose the
            profile to apply when PrismForge connects. To apply a profile or save the current lighting, connect first.
          </p>
          {#if !session && store.kind === "Real"}
            <p>Demo profiles show only in demo mode.</p>
          {/if}
        </div>
      {/if}
      {#if store.summary.notices.length > 0}
        <RecoveryNotices notices={store.summary.notices} ondismiss={(notice) => profiles.dismiss(notice)} />
      {/if}

      <ResultPanel outcome={profiles.outcome} {labels} onsavenew={() => nameField?.focus()} />
      <ImportResult report={profiles.imported} problem={profiles.importProblem} />

      {#key profiles.scope}
        {#if live}
          <SaveCurrent {profiles} bind:field={nameField} />
        {:else}
          <p class="quiet">Connect to save the current lighting as a profile.</p>
        {/if}
      {/key}

      {#if details}
        {#key profiles.scope}
          <ProfileArticle {profiles} {details} summary={store.summary} {labels} {live} {applying} {session} />
        {/key}
      {:else if profiles.detailsProblem}
        <p class="quiet" role="alert">{failureText(profiles.detailsProblem)}</p>
      {:else if store.summary.profiles.length === 0}
        <div class="empty">
          <p class="lead">No profiles yet.</p>
          <p>Set up effects or hardware modes on the Effects screen, then save the current lighting here.</p>
        </div>
      {:else if profiles.selected === null}
        <p class="quiet">Choose a profile on the left to see what it holds.</p>
      {/if}
    {/if}
  </div>
</div>

<style>
  .screen {
    display: grid;
    grid-template-columns: minmax(260px, 320px) 1fr;
    min-height: 0;
    height: 100%;
  }

  .workspace {
    overflow-y: auto;
    padding: 28px 36px 48px;
    display: grid;
    gap: 24px;
    align-content: start;
    max-width: 920px;
  }

  .mode {
    display: grid;
    gap: 4px;
    padding: 10px 14px;
    border-left: 3px solid var(--line);
    background: var(--ink-1);
    color: var(--text-dim);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    max-width: 80ch;
  }

  .quiet {
    color: var(--text-dim);
  }

  .empty {
    display: grid;
    gap: 4px;
    color: var(--text-dim);
  }

  .lead {
    color: var(--text);
    font-weight: 600;
  }

  .notice {
    display: grid;
    gap: 8px;
    justify-items: start;
  }
</style>
