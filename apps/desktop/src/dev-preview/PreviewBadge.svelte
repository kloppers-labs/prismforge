<script lang="ts">
  import { SCENARIOS } from "./scenarios";

  let { current }: { current: string } = $props();

  let open = $state(false);

  function choose(event: Event) {
    const url = new URL(window.location.href);
    url.searchParams.set("state", (event.currentTarget as HTMLSelectElement).value);
    window.location.assign(url);
  }
</script>

<aside class="preview" aria-label="Interface preview">
  <button type="button" class="small" aria-expanded={open} onclick={() => (open = !open)}>
    Interface preview: simulated data
  </button>
  {#if open}
    <div class="panel">
      <p>This page shows the PrismForge interface with recorded test data. It is not the app, and nothing reaches hardware.</p>
      <label>
        <span>State</span>
        <select value={current} onchange={choose}>
          {#each Object.entries(SCENARIOS) as [key, scenario] (key)}
            <option value={key}>{scenario.label}</option>
          {/each}
        </select>
      </label>
    </div>
  {/if}
</aside>

<style>
  .preview {
    position: fixed;
    right: var(--space-2);
    bottom: var(--space-2);
    z-index: calc(var(--z-popover) + 1);
    display: grid;
    justify-items: end;
    gap: var(--space-2);
  }

  .preview > button {
    background: var(--working);
    border-color: var(--working);
    color: var(--surface-0);
  }

  .panel {
    display: grid;
    gap: var(--space-2);
    width: 300px;
    padding: var(--space-3);
    border: 2px solid var(--working);
    border-radius: var(--radius-m);
    background: var(--surface-1);
    box-shadow: var(--shadow-2);
  }

  label {
    display: grid;
    gap: var(--space-1);
  }

  select {
    font: inherit;
    color: var(--text);
    background: var(--surface-0);
    border: 1px solid var(--line-control);
    border-radius: var(--radius-s);
    padding: 6px;
  }
</style>
