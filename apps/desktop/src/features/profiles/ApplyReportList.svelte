<script lang="ts">
  import type { ApplyReport } from "../../lib/api/bindings/ApplyReport";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import { deviceLabel, frameRateText, itemText, outcomeText, type Labels } from "./present";

  let { report, labels }: { report: ApplyReport; labels: Labels } = $props();
</script>

<ul class="report">
  {#each report.items as entry, index (index)}
    <li>
      <ToneIcon tone={entry.outcome === "Applied" ? "ok" : "warning"} />
      <span>
        <strong>{deviceLabel(labels, entry.target, entry.label)}</strong>: {itemText(labels, entry.item, entry.target)}.
        {outcomeText(labels, entry.outcome)}
      </span>
    </li>
  {/each}
</ul>
<p class="rate">{frameRateText(report)}</p>

<style>
  .report {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }

  li {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }

  li :global(svg) {
    flex: none;
    margin-top: 3px;
  }

  .rate {
    color: var(--text-dim);
  }
</style>
