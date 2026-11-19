<script lang="ts">
  import { failureText } from "../errors";
  import Notice from "../Notice.svelte";
  import type { AppState } from "../state/app.svelte";
  import { summarise } from "../../features/connection/wording";
  import ApplyReportList from "../../features/profiles/ApplyReportList.svelte";
  import { connectResultText, fullyApplied, type Labels } from "../../features/profiles/present";
  import TrayStopReport from "../../features/settings/TrayStopReport.svelte";
  import { lifecycleFailureText, trayStopTitle } from "../../features/settings/wording";
  import { activeNotices } from "./notices";

  let { app, labels, region = $bindable() }: { app: AppState; labels: Labels; region?: HTMLElement } = $props();

  let kinds = $derived(activeNotices(app));
  let expanded = $state(false);
  let shown = $derived(expanded ? kinds : kinds.slice(0, 1));

  $effect(() => {
    if (kinds.length <= 1) {
      expanded = false;
    }
  });
</script>

{#snippet applyReport()}
  {@const outcome = app.view.connect_apply?.outcome}
  {#if outcome && "Applied" in outcome}
    <ApplyReportList report={outcome.Applied.report} {labels} />
  {/if}
{/snippet}

{#if kinds.length > 0}
  <section class="slot" aria-label="Notices" bind:this={region}>
    {#each shown as kind (kind)}
      {#if kind === "problem" && app.problem}
        <Notice tone="problem" role="alert" title={failureText(app.problem)} ondismiss={() => (app.problem = null)} />
      {:else if kind === "lifecycle" && app.lifecycle.failure && (app.lifecycle.failure.key === "close" || app.lifecycle.failure.key === "quit")}
        <Notice
          tone="problem"
          role="alert"
          title={lifecycleFailureText(app.lifecycle.failure.key, failureText(app.lifecycle.failure.error))}
          ondismiss={() => (app.lifecycle.failure = null)}
        />
      {:else if kind === "connection" && app.session}
        {@const summary = summarise(app.session)}
        <Notice tone={summary.tone} label="Connection status" title={summary.reasons[0] ?? summary.label}>
          {#each summary.reasons.slice(1) as reason (reason)}
            <p class="line">{reason}</p>
          {/each}
        </Notice>
      {:else if kind === "repaired"}
        <Notice tone="warning" label="Repaired files" title="PrismForge repaired stored files. Open Profiles to read what happened and what to check.">
          {#snippet actions()}
            <button type="button" class="small" onclick={() => app.openProfiles()}>Open Profiles</button>
          {/snippet}
        </Notice>
      {:else if kind === "trayStop" && app.lifecycle.trayStop}
        {@const report = app.lifecycle.trayStop}
        {@const title = trayStopTitle(report)}
        <Notice
          tone={report.stops.some((stop) => "Failed" in stop.result) ? "warning" : "ok"}
          label={title}
          {title}
          ondismiss={() => app.lifecycle.dismissStop()}
        >
          <TrayStopReport {report} session={app.session} label={(device, reported) => app.names.given(device) ?? reported} />
        </Notice>
      {:else if kind === "connectApply" && app.view.connect_apply}
        {@const applied = app.view.connect_apply}
        {@const outcome = applied.outcome}
        <Notice
          tone={"Applied" in outcome ? (fullyApplied(outcome.Applied.report) ? "ok" : "warning") : "problem"}
          label="Applied on connect"
          title={connectResultText(applied)}
          ondismiss={() => (app.connectApplyDismissed = true)}
          details={"Applied" in outcome ? applyReport : undefined}
        />
      {/if}
    {/each}
    {#if kinds.length > 1}
      <button type="button" class="small quiet toggle" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
        {expanded ? "Show fewer notices" : `Show all ${kinds.length} notices`}
      </button>
    {/if}
  </section>
{/if}

<style>
  .slot {
    display: grid;
    gap: 1px;
    max-height: 40vh;
    overflow-y: auto;
    background: var(--line-subtle);
    border-bottom: 1px solid var(--line-subtle);
  }

  .line {
    color: var(--text-dim);
  }

  .toggle {
    justify-self: stretch;
    border-radius: 0;
    background: var(--surface-1);
    text-align: left;
    padding-left: calc(var(--space-3) + 3px);
  }
</style>
