<script lang="ts">
  import type { ConnectApply } from "../../lib/api/bindings/ConnectApply";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import ApplyReportList from "./ApplyReportList.svelte";
  import { connectResultText, fullyApplied, type Labels } from "./present";

  let { applied, labels, ondismiss }: { applied: ConnectApply; labels: Labels; ondismiss: () => void } = $props();

  let outcome = $derived(applied.outcome);
</script>

<div class="notice" role="status" aria-label="Applied on connect">
  {#if "Applied" in outcome}
    {@const report = outcome.Applied.report}
    <p class="headline">
      <ToneIcon tone={fullyApplied(report) ? "ok" : "warning"} />
      {connectResultText(applied)}
    </p>
    <ApplyReportList {report} {labels} />
  {:else}
    <p class="headline">
      <ToneIcon tone="problem" />
      {connectResultText(applied)}
    </p>
  {/if}
  <div><button type="button" class="small" onclick={ondismiss}>Dismiss</button></div>
</div>

<style>
  .notice {
    display: grid;
    gap: 8px;
    padding: 10px 20px;
    border-bottom: 1px solid var(--line);
    background: var(--ink-1);
  }

  .headline {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    font-weight: 600;
  }

  .headline :global(svg) {
    flex: none;
    margin-top: 4px;
  }

  .small {
    padding: 3px 10px;
    font-size: 13px;
  }
</style>
