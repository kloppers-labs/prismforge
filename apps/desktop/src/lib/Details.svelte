<script lang="ts">
  import type { Snippet } from "svelte";

  let { summary = "Details", open = $bindable(false), children }: { summary?: string; open?: boolean; children: Snippet } = $props();
</script>

<details class="details" bind:open>
  <summary>{summary}</summary>
  <div class="body">{@render children()}</div>
</details>

<style>
  .details {
    max-width: 72ch;
  }

  summary {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    min-height: var(--target-min);
    color: var(--text-dim);
    font-size: var(--font-s);
    cursor: pointer;
    list-style: none;
    border-radius: var(--radius-xs);
  }

  summary::-webkit-details-marker {
    display: none;
  }

  summary::before {
    content: "";
    width: 6px;
    height: 6px;
    border-right: 1.5px solid currentColor;
    border-bottom: 1.5px solid currentColor;
    transform: rotate(-45deg);
    transition: transform var(--dur-fast) var(--ease-out);
  }

  details[open] > summary::before {
    transform: rotate(45deg);
  }

  summary:hover {
    color: var(--text);
  }

  .body {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-2) 0 var(--space-1) var(--space-4);
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
