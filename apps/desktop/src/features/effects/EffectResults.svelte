<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { serverName } from "../connection/wording";
  import FailureNotice from "./FailureNotice.svelte";
  import { stopBlocked, stopText } from "./present";
  import type { EffectsState } from "./state.svelte";

  let { effects, session, blocked, held }: { effects: EffectsState; session: SessionSnapshot; blocked: string | null; held: string | null } = $props();

  let feedback = $derived(effects.feedback);
  let server = $derived(serverName(session));
  let stopReason = $derived(held ?? (effects.target ? stopBlocked(session, effects.target) : null));
  let box: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (feedback) {
      box?.scrollIntoView?.({ block: "nearest" });
    }
  });
</script>

<div class="results" bind:this={box}>
  {#if stopReason && blocked === null}
    <p id="stop-blocked" class="note">Stop is off. {stopReason}</p>
  {/if}
  {#if feedback?.kind === "failed"}
    <FailureNotice error={feedback.error} />
  {/if}
  <div class="result" role="status" aria-label="Effect result">
    {#if feedback?.kind === "applied"}
      <p class="outcome">PrismForge accepted {feedback.label} for {feedback.targetLabel}.</p>
    {:else if feedback?.kind === "stopped"}
      {@const text = stopText(feedback.outcome, server)}
      <div class="outcome" class:unsettled={!text.settled}>
        <p class="headline">{text.headline}</p>
        {#each text.details as detail (detail)}
          <p>{detail}</p>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .results {
    display: grid;
    gap: var(--space-2);
  }

  .results:has(.result:empty):not(:has(.note, [role="alert"])) {
    display: none;
  }

  .note {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .result:empty {
    display: none;
  }

  .outcome {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--ok);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }

  .outcome.unsettled {
    border-left-color: var(--warning);
  }

  .headline {
    color: var(--text);
    font-weight: var(--weight-strong);
  }
</style>
