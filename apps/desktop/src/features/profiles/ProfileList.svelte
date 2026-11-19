<script lang="ts">
  import type { StoreView } from "../../lib/api/bindings/StoreView";
  import { contents, found, unsavedText } from "./present";
  import type { ProfilesState } from "./state.svelte";

  let {
    profiles,
    store,
    live,
    onsave,
    saveButton = $bindable(),
  }: { profiles: ProfilesState; store: StoreView | null; live: boolean; onsave: () => void; saveButton?: HTMLButtonElement } = $props();

  let summary = $derived(store?.summary ?? null);
  let rows: HTMLButtonElement[] = $state([]);
  let firstId = $derived(summary?.profiles[0]?.id ?? null);
  let anchor = $derived(summary?.profiles.some((profile) => profile.id === profiles.selected) ? profiles.selected : firstId);

  function move(event: KeyboardEvent, index: number) {
    const count = summary?.profiles.length ?? 0;
    const next = { ArrowDown: index + 1, ArrowUp: index - 1, Home: 0, End: count - 1 }[event.key];
    if (next === undefined || !summary) {
      return;
    }
    event.preventDefault();
    const at = Math.min(Math.max(next, 0), count - 1);
    const profile = summary.profiles[at];
    if (profile) {
      void profiles.select(profile.id);
      rows[at]?.focus();
    }
  }
</script>

<nav class="list" aria-labelledby="profiles-title">
  <h1 id="profiles-title" class="title">Profiles</h1>
  <div class="head">
    {#if live}
      <button type="button" class="small" bind:this={saveButton} disabled={store === null || profiles.working !== null} onclick={onsave}>Save current lighting…</button>
    {/if}
    <button type="button" class="small quiet" disabled={store === null || profiles.working === "import"} onclick={() => profiles.importProfile()}>
      {profiles.working === "import" ? "Importing…" : "Import a profile"}
    </button>
  </div>
  {#if summary}
    <ul>
      {#each summary.profiles as profile, index (profile.id)}
        {@const active = summary.active === profile.id}
        {@const missing = profile.resolution ? profile.devices - profile.resolution.resolved : 0}
        <li>
          <button
            type="button"
            class="pick"
            bind:this={rows[index]}
            tabindex={anchor === profile.id ? 0 : -1}
            aria-current={profiles.selected === profile.id ? "true" : undefined}
            onclick={() => profiles.select(profile.id)}
            onkeydown={(event) => move(event, index)}
          >
            <span class="pick-name">{profile.name}</span>
            <span class="pick-meta">{contents(profile)}</span>
            <span class="pick-meta">{found(profile.devices, profile.resolution)}</span>
            {#if active || missing > 0 || summary.apply_on_connect === profile.id}
              <span class="tags">
                {#if active}<span class="tag good">Active in this session</span>{/if}
                {#if active && summary.unsaved_changes !== 0}<span class="tag">{unsavedText(summary.unsaved_changes)}</span>{/if}
                {#if missing > 0}<span class="tag caution">{missing} not found today</span>{/if}
                {#if summary.apply_on_connect === profile.id}<span class="tag">Applies on connect</span>{/if}
              </span>
            {/if}
          </button>
        </li>
      {/each}
      {#each summary.unreadable as locked (locked.id)}
        <li class="locked">A profile file needs a newer PrismForge. This version leaves it as it is.</li>
      {/each}
    </ul>
  {/if}
</nav>

<style>
  .list {
    border-right: 1px solid var(--line-subtle);
    background: var(--surface-1);
    overflow-y: auto;
    padding: var(--space-4) var(--space-3);
    display: grid;
    gap: var(--space-3);
    align-content: start;
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
    padding: 0 var(--space-2);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding: 0 var(--space-2);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-3);
    margin-top: var(--space-1);
  }

  .locked {
    padding: var(--space-2) var(--space-3);
    font-size: var(--font-s);
    color: var(--text-dim);
    border: 1px dashed var(--line-control);
    border-radius: var(--radius-s);
  }
</style>
