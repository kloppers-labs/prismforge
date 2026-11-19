<script lang="ts">
  import type { OnConnect } from "../../lib/api/bindings/OnConnect";
  import { failureText } from "../../lib/errors";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import ApplyReportList from "./ApplyReportList.svelte";
  import ChangeList from "./ChangeList.svelte";
  import { applyHeadline, count, fullyApplied, waitingText, type Labels } from "./present";
  import type { Outcome } from "./state.svelte";

  let {
    outcome,
    labels,
    onConnect,
    onsavenew,
  }: { outcome: Outcome | null; labels: Labels; onConnect: OnConnect; onsavenew: () => void } = $props();

  let problem = $derived(outcome?.kind === "failed");
</script>

<div class="result" class:problem role="status" aria-label="Profile result">
  {#if outcome?.kind === "applied"}
    {#if outcome.outcome === "WaitingForConnection"}
      <p class="headline">{waitingText(outcome.name, onConnect, labels.session)}</p>
    {:else}
      {@const report = outcome.outcome.Applied}
      <p class="headline">
        <ToneIcon tone={fullyApplied(report) ? "ok" : "warning"} />
        {applyHeadline(outcome.name, report)}
      </p>
      <ApplyReportList {report} {labels} />
    {/if}
  {:else if outcome?.kind === "created"}
    <p class="headline">Saved the current lighting as a new profile on this computer. Nothing is saved to the devices.</p>
  {:else if outcome?.kind === "updated"}
    {@const changed = outcome.saved.changes.length + (outcome.saved.frame_rate ? 1 : 0)}
    <p class="headline">Saved {count(changed, "change", "changes")} to {outcome.name} on this computer. Nothing is saved to the devices.</p>
    <ChangeList saved={outcome.saved} {labels} />
  {:else if outcome?.kind === "nothing"}
    <p class="headline">Nothing to save. {outcome.name} already holds every change you made since you applied or saved it.</p>
  {:else if outcome?.kind === "notActive"}
    <p class="headline">
      {outcome.name} is not the profile applied in this session, so PrismForge cannot tell which changes belong to it.
    </p>
    <p>PrismForge links your changes to a profile only from the moment you apply or save it, until you disconnect, switch between demo mode and OpenRGB, or close PrismForge. You can keep the current lighting as a new profile instead.</p>
    <div><button type="button" onclick={onsavenew}>Save as a new profile</button></div>
  {:else if outcome?.kind === "exported"}
    <p class="headline">Exported {outcome.name}.</p>
    {#if outcome.report.omitted_fields > 0}
      <p>{count(outcome.report.omitted_fields, "field", "fields")} from a newer PrismForge were left out, because they could hold private data.</p>
    {/if}
  {:else if outcome?.kind === "exportCancelled"}
    <p>Export cancelled. Nothing was written.</p>
  {:else if outcome?.kind === "deleted"}
    <p class="headline">Deleted {outcome.name} from this computer.</p>
  {:else if outcome?.kind === "duplicated"}
    <p class="headline">Made a copy of {outcome.name}.</p>
  {:else if outcome?.kind === "failed"}
    <p class="headline"><ToneIcon tone="problem" /> {failureText(outcome.error)}</p>
  {/if}
</div>

<style>
  .result {
    display: grid;
    gap: 8px;
  }

  .result:empty {
    display: none;
  }

  .headline {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: var(--text);
    font-weight: 600;
  }

  .headline :global(svg) {
    flex: none;
    margin-top: 4px;
  }

  .problem .headline {
    color: var(--problem);
  }

  p {
    max-width: 72ch;
  }
</style>
