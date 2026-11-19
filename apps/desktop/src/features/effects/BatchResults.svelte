<script lang="ts">
  import { untrack } from "svelte";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { serverName } from "../connection/wording";
  import { summarize } from "./batch";
  import FailureNotice from "./FailureNotice.svelte";
  import { targetsKey, type EffectsState } from "./state.svelte";

  let { effects, session, names }: { effects: EffectsState; session: SessionSnapshot; names: Names } = $props();

  let batch = $derived(effects.batch);
  let summary = $derived(batch ? summarize(batch, session, names, serverName(session)) : null);
  let box: HTMLDivElement | undefined = $state();
  let seen = untrack(() => effects.pressed?.count ?? 0);

  $effect(() => {
    const press = effects.pressed;
    if (press === null || press.count === seen) {
      return;
    }
    seen = press.count;
    if (untrack(() => effects.batch) !== null && press.key === untrack(() => targetsKey(effects.targets))) {
      untrack(() => box)?.scrollIntoView?.({ block: "nearest" });
    }
  });
</script>

<div class="batch" bind:this={box}>
  {#if batch?.kind === "failed"}
    <FailureNotice error={batch.error} />
  {/if}
  <div role="status" aria-label="Results for each target">
    {#if summary}
      <section class="report" class:unsettled={!summary.settled} aria-labelledby="batch-headline">
        <p id="batch-headline" class="headline">{summary.headline}</p>
        <ul>
          {#each summary.rows as row (row.key)}
            <li class={row.state.toLowerCase()}><span class="state">{row.state}</span> {row.text}</li>
          {/each}
        </ul>
        {#if summary.retry > 0}
          <button type="button" class="small" disabled={effects.working !== null || effects.blocked !== null} onclick={() => effects.retry()}>
            Retry {summary.retry === 1 ? "1 target" : `${summary.retry} targets`}
          </button>
        {/if}
      </section>
    {/if}
  </div>
</div>

<style>
  .batch {
    display: grid;
    gap: var(--space-2);
  }

  .batch:not(:has(.report, [role="alert"])) {
    display: none;
  }

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

  .skipped .state,
  .failed .state {
    color: var(--warning);
  }
</style>
