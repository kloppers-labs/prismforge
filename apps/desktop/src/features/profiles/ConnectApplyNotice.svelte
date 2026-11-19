<script lang="ts">
  import type { ConnectApply } from "../../lib/api/bindings/ConnectApply";
  import { failureText } from "../../lib/errors";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import ApplyReportList from "./ApplyReportList.svelte";
  import { connectHeadline, fullyApplied, type Labels } from "./present";

  let { applied, labels, ondismiss }: { applied: ConnectApply; labels: Labels; ondismiss: () => void } = $props();

  let outcome = $derived(applied.outcome);
</script>

<div class="notice" role="status" aria-label="Applied on connect">
  {#if "Applied" in outcome}
    {@const report = outcome.Applied.report}
    <p class="headline">
      <ToneIcon tone={fullyApplied(report) ? "ok" : "warning"} />
      {connectHeadline(applied.name, report)}
    </p>
    <ApplyReportList {report} {labels} />
  {:else}
    <p class="headline">
      <ToneIcon tone="problem" />
      PrismForge could not apply {applied.name} when it connected. {failureText(outcome.Failed.error)}
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
