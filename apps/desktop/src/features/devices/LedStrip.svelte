<script lang="ts">
  import type { Rgb } from "../../lib/api/bindings/Rgb";
  import { ledCount } from "./present";

  let { colors, available }: { colors: Rgb[]; available: boolean } = $props();

  const LIMIT = 120;
  let shown = $derived(colors.slice(0, LIMIT));
  let hex = (color: Rgb) => `#${[color.r, color.g, color.b].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
  let distinct = $derived([...new Set(colors.map(hex))]);
  let summary = $derived(
    colors.length === 0
      ? "No LED colours reported."
      : distinct.length === 1
        ? `${ledCount(colors.length)}, all ${distinct[0]}.`
        : `${ledCount(colors.length)} in ${distinct.length} colours.`,
  );
</script>

{#if available}
  <div class="strip true-color" aria-hidden="true">
    {#each shown as color, index (index)}
      <span style:background-color="rgb({color.r} {color.g} {color.b})"></span>
    {/each}
  </div>
  <p class="summary">{summary}{colors.length > LIMIT ? ` The strip shows the first ${LIMIT}.` : ""}</p>
{:else}
  <div class="strip unknown" aria-hidden="true"></div>
  <p class="summary">Not reported now. {summary}</p>
{/if}

<style>
  .strip {
    display: flex;
    gap: 2px;
    height: var(--space-4);
  }

  .unknown {
    border-radius: var(--radius-xs);
    background: repeating-linear-gradient(45deg, var(--surface-2) 0 6px, var(--surface-3) 6px 12px);
    max-width: 320px;
  }

  span {
    flex: 1 1 0;
    min-width: 2px;
    max-width: var(--space-5);
    border-radius: 2px;
    box-shadow: inset 0 0 0 1px var(--line-subtle);
  }

  .summary {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
