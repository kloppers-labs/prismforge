<script lang="ts">
  import { untrack } from "svelte";
  import Notice from "../Notice.svelte";
  import type { AppState } from "../state/app.svelte";
  import { summarise } from "../../features/connection/wording";
  import ApplyReportList from "../../features/profiles/ApplyReportList.svelte";
  import type { Labels } from "../../features/profiles/present";
  import TrayStopReport from "../../features/settings/TrayStopReport.svelte";
  import { activeNotices, NoticeOrder, type NoticeItem } from "./notices";

  let { app, labels, region = $bindable() }: { app: AppState; labels: Labels; region?: HTMLElement } = $props();

  const queue = new NoticeOrder();
  let notices = $state<NoticeItem[]>([]);
  let expanded = $state(false);
  let shown = $derived(expanded ? notices : notices.slice(0, 1));
  let announced = $state<{ id: number; text: string; urgent: boolean } | null>(null);

  $effect(() => {
    const items = activeNotices(app);
    untrack(() => {
      const { sorted, arrived } = queue.order(items);
      notices = sorted;
      if (sorted.length <= 1) {
        expanded = false;
      }
      const visible = new Set((expanded ? sorted : sorted.slice(0, 1)).map((notice) => notice.kind));
      const hidden = arrived.filter((notice) => !visible.has(notice.kind));
      if (hidden.length > 0) {
        announced = {
          id: (announced?.id ?? 0) + 1,
          text: `${sorted.length} notices. New: ${hidden.map((notice) => notice.title).join(" ")}`,
          urgent: hidden.some((notice) => notice.tone === "problem"),
        };
      }
    });
  });

  function dismiss(notice: NoticeItem) {
    if (notice.kind === "problem") {
      app.dismissProblem();
    } else if (notice.kind === "lifecycle") {
      app.lifecycle.failure = null;
    } else if (notice.kind === "trayStop") {
      app.lifecycle.dismissStop();
    } else if (notice.kind === "connectApply") {
      app.connectApplyDismissed = true;
    }
  }
</script>

{#snippet applyReport()}
  {@const outcome = app.view.connect_apply?.outcome}
  {#if outcome && "Applied" in outcome}
    <ApplyReportList report={outcome.Applied.report} {labels} />
  {/if}
{/snippet}

{#snippet trayReport()}
  {#if app.lifecycle.trayStop}
    <TrayStopReport report={app.lifecycle.trayStop} session={app.session} label={(device, reported) => app.names.given(device) ?? reported} />
  {/if}
{/snippet}

{#snippet announcement(urgent: boolean)}
  {#if announced && announced.urgent === urgent}
    {#key announced.id}<span>{announced.text}</span>{/key}
  {/if}
{/snippet}

<p class="visually-hidden" role="status" aria-label="New notices">{@render announcement(false)}</p>
<p class="visually-hidden" aria-live="assertive" aria-atomic="true">{@render announcement(true)}</p>

{#if notices.length > 0}
  <section class="slot" aria-label="Notices" tabindex="-1" bind:this={region}>
    <div class="list">
    {#each shown as notice (notice.kind)}
      {#if notice.kind === "problem" || notice.kind === "lifecycle"}
        <Notice tone="problem" role="alert" title={notice.title} ondismiss={() => dismiss(notice)} />
      {:else if notice.kind === "connection" && app.session}
        <Notice tone={notice.tone} label="Connection status" title={notice.title}>
          {#each summarise(app.session).reasons.slice(1) as reason (reason)}
            <p class="line">{reason}</p>
          {/each}
        </Notice>
      {:else if notice.kind === "repaired"}
        <Notice tone="warning" label="Repaired files" title={notice.title}>
          {#snippet actions()}
            <button type="button" class="small" onclick={() => app.openProfiles()}>Open Profiles</button>
          {/snippet}
        </Notice>
      {:else if notice.kind === "trayStop"}
        {#if notice.tone === "warning"}
          <Notice tone="warning" label={notice.title} title={notice.title} ondismiss={() => dismiss(notice)}>
            {@render trayReport()}
          </Notice>
        {:else}
          <Notice
            tone="ok"
            label={notice.title}
            title={notice.title}
            details={app.lifecycle.trayStop && app.lifecycle.trayStop.stops.length > 0 ? trayReport : undefined}
            ondismiss={() => dismiss(notice)}
          />
        {/if}
      {:else if notice.kind === "connectApply"}
        {@const outcome = app.view.connect_apply?.outcome}
        <Notice
          tone={notice.tone}
          label="Applied on connect"
          title={notice.title}
          details={outcome && "Applied" in outcome ? applyReport : undefined}
          ondismiss={() => dismiss(notice)}
        />
      {/if}
    {/each}
    </div>
    {#if notices.length > 1}
      <button type="button" class="small quiet toggle" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
        {expanded ? "Show fewer notices" : `Show all ${notices.length} notices`}
      </button>
    {/if}
  </section>
{/if}

<style>
  .slot {
    display: grid;
    outline-offset: -2px;
    border-bottom: 1px solid var(--line-subtle);
  }

  .list {
    display: grid;
    gap: 1px;
    max-height: 40vh;
    overflow-y: auto;
    background: var(--line-subtle);
  }

  .line {
    color: var(--text-dim);
  }

  .toggle {
    justify-self: stretch;
    border-top: 1px solid var(--line-subtle);
    border-radius: 0;
    background: var(--surface-1);
    text-align: left;
    padding-left: calc(var(--space-3) + 3px);
  }
</style>
