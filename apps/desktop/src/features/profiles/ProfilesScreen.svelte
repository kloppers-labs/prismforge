<script lang="ts">
  import { untrack } from "svelte";
  import { failureText } from "../../lib/errors";
  import type { AppState } from "../../lib/state/app.svelte";
  import { isLive } from "../devices/present";
  import { applyingText } from "../effects/present";
  import Notice from "../../lib/Notice.svelte";
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
  let saving = $state(false);
  let saveButton: HTMLButtonElement | undefined = $state();

  let connected = $derived(session !== null);

  $effect(() => {
    if (connected) {
      untrack(() => app.effects.load());
    }
  });
</script>

<div class="two-pane">
  <ProfileList {profiles} {store} {live} onsave={() => (saving = true)} bind:saveButton />
  <div class="workspace">
    <div class="inner">
      {#if profiles.storeProblem}
        <Notice tone="problem" role="alert" title="PrismForge could not read the profiles.">
          <p>{failureText(profiles.storeProblem)}</p>
          {#snippet actions()}
            <button type="button" class="small" onclick={() => profiles.refresh()}>Try again</button>
          {/snippet}
        </Notice>
      {:else if !store}
        <p class="quiet" role="status">Reading the profiles.</p>
      {:else}
        {#if !live}
          <p class="mode">
            <span>Not connected. You can still rename, duplicate, delete, export and import profiles.</span>
            {#if !session}
              <button type="button" class="small" onclick={() => app.go("devices")}>Connect to OpenRGB</button>
            {/if}
          </p>
        {/if}

        {#if details}
          {#key profiles.scope}
            <ProfileArticle
              {profiles}
              {details}
              summary={store.summary}
              {labels}
              {live}
              {applying}
              {session}
              onConnect={app.view.on_connect}
              onsavenew={() => (saving = true)}
            />
          {/key}
        {:else}
          <ResultPanel outcome={profiles.outcome} {labels} onConnect={app.view.on_connect} onsavenew={() => (saving = true)} />
          {#if profiles.detailsProblem}
            <p class="quiet" role="alert">{failureText(profiles.detailsProblem)}</p>
          {:else if store.summary.profiles.length === 0}
            <div class="empty">
              <p class="lead">No profiles yet.</p>
              <p>Set up effects or hardware modes on the Effects screen, then save the current lighting with the button on the left.</p>
            </div>
          {:else if profiles.selected === null}
            <p class="quiet">Choose a profile on the left to see what it holds.</p>
          {/if}
        {/if}

        <ImportResult report={profiles.imported} problem={profiles.importProblem} />
        {#if store.kind === "Demo"}
          <p class="mode">These are demo profiles. They are kept apart from your real profiles and never apply to real hardware.</p>
        {:else if !session}
          <p class="quiet small-text">Demo profiles show only in demo mode.</p>
        {/if}
        {#if store.summary.notices.length > 0}
          <RecoveryNotices notices={store.summary.notices} ondismiss={(notice) => profiles.dismiss(notice)} />
        {/if}
      {/if}
    </div>
  </div>
</div>

{#if saving && live}
  {#key profiles.scope}
    <SaveCurrent {profiles} opener={saveButton} onclose={() => (saving = false)} />
  {/key}
{/if}

<style>
  .workspace {
    position: relative;
    min-width: 0;
    overflow-y: auto;
  }

  .inner {
    max-width: 920px;
    padding: var(--space-5) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-5);
    align-content: start;
  }

  .mode {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-control);
    background: var(--surface-1);
    color: var(--text-dim);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    max-width: 80ch;
  }

  .quiet {
    color: var(--text-dim);
  }

  .small-text {
    font-size: var(--font-s);
  }

  .empty {
    display: grid;
    gap: var(--space-1);
    color: var(--text-dim);
  }

  .lead {
    color: var(--text);
    font-weight: var(--weight-strong);
  }
</style>
