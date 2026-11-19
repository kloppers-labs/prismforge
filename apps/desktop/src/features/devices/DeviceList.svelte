<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { Names } from "../../lib/names";
  import { distinction, hasDirectMode, isAmbiguous, kindLabel, ledCount } from "./present";

  type Entry = { key: string; device: SessionDevice };

  let {
    entries,
    selected,
    issues,
    names,
    onselect,
  }: { entries: Entry[]; selected: string | null; issues: number; names: Names; onselect: (key: string) => void } = $props();

  let rows: HTMLButtonElement[] = $state([]);
  let apart = $derived(distinction(entries.map((entry) => entry.device), (device) => names.device(device)));

  function move(event: KeyboardEvent, index: number) {
    const next = { ArrowDown: index + 1, ArrowUp: index - 1, Home: 0, End: entries.length - 1 }[event.key];
    if (next === undefined) {
      return;
    }
    event.preventDefault();
    const at = Math.min(Math.max(next, 0), entries.length - 1);
    const entry = entries[at];
    if (entry) {
      onselect(entry.key);
      rows[at]?.focus();
    }
  }
</script>

<nav class="list" aria-labelledby="devices-title">
  <h1 id="devices-title" class="title">Devices <span class="count numeric">{entries.length}</span></h1>
  <ul>
    {#each entries as { key, device }, index (key)}
      <li>
        <button
          type="button"
          class="pick"
          bind:this={rows[index]}
          tabindex={key === selected ? 0 : -1}
          aria-current={key === selected ? "true" : undefined}
          onclick={() => onselect(key)}
          onkeydown={(event) => move(event, index)}
        >
          <span class="pick-name">{names.device(device)}</span>
          {#if apart[index]}<span class="pick-meta">{apart[index]}</span>{/if}
          <span class="pick-meta">{kindLabel(device.descriptor.kind)}, {ledCount(device.descriptor.led_count)}</span>
          {#if !device.available || isAmbiguous(device) || !hasDirectMode(device)}
            <span class="tags">
              {#if !device.available}<span class="tag">Unavailable</span>{/if}
              {#if isAmbiguous(device)}<span class="tag caution">Identical device</span>{/if}
              {#if !hasDirectMode(device)}<span class="tag">No Direct Mode</span>{/if}
            </span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>
  {#if issues > 0}
    <p class="issues">
      {issues === 1 ? "One controller" : `${issues} controllers`} sent data PrismForge could not read and {issues === 1
        ? "is"
        : "are"} not listed. Diagnostics has the details.
    </p>
  {/if}
</nav>

<style>
  .list {
    border-right: 1px solid var(--line-subtle);
    background: var(--surface-1);
    overflow-y: auto;
    padding: var(--space-4) var(--space-3);
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
    padding: 0 var(--space-2) var(--space-3);
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
  }

  .count {
    color: var(--text-dim);
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

  .issues {
    margin: var(--space-4) var(--space-2) 0;
    font-size: var(--font-s);
    color: var(--text-dim);
  }
</style>
