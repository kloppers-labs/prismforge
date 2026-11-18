<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import { hasDirectMode, isAmbiguous, kindLabel, ledCount } from "./present";

  type Entry = { key: string; device: SessionDevice };

  let {
    entries,
    selected,
    issues,
    onselect,
  }: { entries: Entry[]; selected: string | null; issues: number; onselect: (key: string) => void } = $props();
</script>

<nav class="list" aria-labelledby="devices-title">
  <h2 id="devices-title" class="title">Devices <span class="count numeric">{entries.length}</span></h2>
  <ul>
    {#each entries as { key, device } (key)}
      <li>
        <button type="button" aria-current={key === selected ? "true" : undefined} onclick={() => onselect(key)}>
          <span class="name">{device.descriptor.name}</span>
          <span class="meta">{kindLabel(device.descriptor.kind)}, {ledCount(device.descriptor.led_count)}</span>
          {#if !device.available || isAmbiguous(device) || !hasDirectMode(device)}
            <span class="tags">
              {#if !device.available}<span class="tag unavailable">Unavailable</span>{/if}
              {#if isAmbiguous(device)}<span class="tag identical">Identical device</span>{/if}
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
    border-right: 1px solid var(--line);
    background: var(--ink-1);
    overflow-y: auto;
    padding: 16px 12px;
  }

  .title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-dim);
    padding: 0 8px 10px;
    display: flex;
    gap: 8px;
    align-items: baseline;
  }

  .count {
    color: var(--text-faint);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  li button {
    width: 100%;
    display: grid;
    gap: 2px;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    padding: 9px 10px 9px 14px;
    position: relative;
    font-weight: 400;
  }

  li button:hover:not(:disabled) {
    background: var(--ink-2);
    border-color: transparent;
  }

  li button[aria-current="true"] {
    background: var(--ink-2);
    border-color: var(--line);
  }

  li button[aria-current="true"]::before {
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
    font-weight: 600;
  }

  .meta {
    color: var(--text-dim);
    font-size: 13px;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }

  .tag {
    font-size: 12px;
    padding: 0 6px;
    border-radius: 4px;
    border: 1px solid var(--line);
    color: var(--text-dim);
  }

  .unavailable {
    border-style: dashed;
  }

  .identical {
    border-color: #6b5a33;
    color: var(--warning);
  }

  .issues {
    margin: 16px 8px 0;
    font-size: 13px;
    color: var(--text-dim);
  }
</style>
