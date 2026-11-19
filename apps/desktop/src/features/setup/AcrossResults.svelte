<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { SetupOutputStatus } from "../../lib/api/bindings/SetupOutputStatus";
  import type { Names } from "../../lib/names";
  import { serverName } from "../connection/wording";
  import { labelOf, summarize } from "../effects/batch";
  import FailureNotice from "../effects/FailureNotice.svelte";
  import type { AcrossState } from "./across.svelte";
  import { listText, nameOf, setupProblemText } from "./present";
  import type { SetupState } from "./state.svelte";

  let { across, setup, session, names }: { across: AcrossState; setup: SetupState; session: SessionSnapshot; names: Names } = $props();

  let result = $derived(across.result);
  let summary = $derived.by(() => {
    if (!result || result.kind === "failed") {
      return null;
    }
    const batch = result.kind === "applied" ? { kind: "applied" as const, label: result.label, instance: result.instance, report: result.report.report } : { kind: "stopped" as const, all: false, report: result.report };
    return summarize(batch, session, names, serverName(session));
  });

  function excludedText(status: SetupOutputStatus): string {
    const where = status.zone_name ? `${status.zone_name} on ${status.label}` : status.label;
    const output = setup.draft.outputs.find((each) => each.id === status.output);
    const holds = output ? ` (${listText(output.branches.flat().map((id) => nameOf(setup.draft, id)))})` : "";
    const why =
      status.problem !== null
        ? setupProblemText(status.problem, setup.draft, names, session)
        : `This zone is no longer ${status.zone_name ?? "the one you placed"}, so its parts may not fit. Place it again on the zone it is on now.`;
    return `${where}${holds}: not sent. ${why}`;
  }
</script>

{#if result?.kind === "failed"}
  <FailureNotice error={result.error} />
{/if}
<div role="status" aria-label="Results for each target">
  {#if summary && result}
    <section class="report" class:unsettled={!summary.settled || (result.kind === "applied" && result.report.excluded.length > 0)} aria-labelledby="across-headline">
      <p id="across-headline" class="headline">{summary.headline}</p>
      <ul>
        {#each summary.rows as row (row.key)}
          <li><span class="state" class:warn={row.state === "Skipped" || row.state === "Failed"}>{row.state}</span> {row.text}</li>
        {/each}
        {#if result.kind === "applied"}
          {#each result.report.excluded as status (status.output)}
            <li><span class="state warn">Not sent</span> {excludedText(status)}</li>
          {/each}
          {#each result.report.not_placed as dark (`${dark.target.device}|${dark.target.zone}`)}
            <li><span class="state">Dark</span> {labelOf(session, names, dark.target)}: {dark.leds} of {dark.of} LEDs have no place and are sent black.</li>
          {/each}
        {/if}
      </ul>
      {#if summary.retry > 0}
        <button type="button" class="small" disabled={across.working !== null || across.blocked !== null} onclick={() => across.retry()}>
          Retry {summary.retry === 1 ? "1 target" : `${summary.retry} targets`}
        </button>
      {/if}
    </section>
  {/if}
</div>

<style>
  .report {
    display: grid;
    gap: var(--space-2);
    justify-items: start;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--ok);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }

  .report.unsettled {
    border-left-color: var(--warning);
  }

  .headline {
    color: var(--text);
    font-weight: var(--weight-strong);
  }

  ul {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .state {
    display: inline-block;
    min-width: 5.5em;
    font-size: var(--font-xs);
    font-weight: var(--weight-heading);
    color: var(--text);
  }

  .state.warn {
    color: var(--warning);
  }
</style>
