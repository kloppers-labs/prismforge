<script lang="ts">
  import type { StoreView } from "../../lib/api/bindings/StoreView";
  import { contents, found } from "./present";
  import type { ProfilesState } from "./state.svelte";

  let { profiles, store }: { profiles: ProfilesState; store: StoreView | null } = $props();

  let summary = $derived(store?.summary ?? null);

  function unsavedTag(count: number | null): string | null {
    if (count === null) {
      return "Unsaved changes: unknown";
    }
    return count > 0 ? `Unsaved changes: ${count}` : null;
  }
</script>

<nav class="list" aria-label="Profiles">
  <div class="head">
    <h2 class="title">Profiles</h2>
    <button type="button" class="small" disabled={store === null || profiles.working === "import"} onclick={() => profiles.importProfile()}>
      {profiles.working === "import" ? "Importing" : "Import a profile"}
    </button>
  </div>
  {#if summary}
    <ul>
      {#each summary.profiles as profile (profile.id)}
        {@const active = summary.active === profile.id}
        {@const unsaved = active ? unsavedTag(summary.unsaved_changes) : null}
        <li>
          <button type="button" aria-current={profiles.selected === profile.id ? "true" : undefined} onclick={() => profiles.select(profile.id)}>
            <span class="name">{profile.name}</span>
            <span class="meta">{contents(profile)}</span>
            <span class="meta">{found(profile.devices, profile.resolution)}</span>
            {#if active || summary.apply_on_connect === profile.id}
              <span class="tags">
                {#if active}<span class="tag active">Active in this session</span>{/if}
                {#if unsaved}<span class="tag">{unsaved}</span>{/if}
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
    padding: 16px 12px;
    display: grid;
    gap: 10px;
    align-content: start;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 0 8px;
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
  }

  .small {
    padding: 3px 10px;
    font-size: var(--font-s);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  li > button {
    width: 100%;
    display: grid;
    gap: 2px;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    padding: 9px 10px 9px 14px;
    position: relative;
    font-weight: var(--weight-regular);
  }

  li > button:hover:not(:disabled) {
    background: var(--surface-2);
    border-color: transparent;
  }

  li > button[aria-current="true"] {
    background: var(--surface-2);
    border-color: var(--line-subtle);
  }

  li > button[aria-current="true"]::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 2px;
    background: var(--spectrum);
    background-size: 100% 600%;
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .meta {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }

  .tag {
    font-size: var(--font-xs);
    padding: 0 6px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--line-subtle);
    color: var(--text-dim);
  }

  .tag.active {
    color: var(--text);
    border-color: var(--ok);
  }

  .locked {
    padding: 9px 10px 9px 14px;
    font-size: var(--font-s);
    color: var(--text-dim);
    border: 1px dashed var(--line-subtle);
    border-radius: var(--radius-s);
  }
</style>
