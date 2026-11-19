<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Details from "../../lib/Details.svelte";
  import type { Names } from "../../lib/names";
  import { serverName } from "../connection/wording";
  import { kindLabel, ledCount } from "../devices/present";
  import FailureNotice from "./FailureNotice.svelte";
  import { conditionText, editMode, outputOf, stopBlocked, stopText, targetLeds, targetName } from "./present";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    session,
    device,
    zone,
    names,
    blocked,
    held,
    closing,
  }: {
    effects: EffectsState;
    session: SessionSnapshot;
    device: SessionDevice;
    zone: number | null;
    names: Names;
    blocked: string | null;
    held: string | null;
    closing: string;
  } = $props();

  let target = $derived(effects.target);
  let running = $derived(effects.running);
  let runningLabel = $derived(effects.catalog?.effects.find((effect) => effect.id === running?.effect.effect)?.label ?? null);
  let name = $derived(targetName(names, device, zone));
  let mode = $derived(
    editMode({
      blocked,
      live: effects.canAutoApply,
      chosen: effects.effect,
      running: running?.effect.effect ?? null,
      runningLabel,
      target: name,
    }),
  );
  let stopReason = $derived(held ?? (target ? stopBlocked(session, target) : null));
  let feedback = $derived(effects.feedback);
  let server = $derived(serverName(session));
  let output = $derived(target ? outputOf(session, target.device) : null);
  let condition = $derived(conditionText(session, output));
</script>

<section class="bar" class:live={mode.name === "Live"} aria-label="Effect actions">
  <div class="title">
    <h2 id="target-title">{names.device(device)}</h2>
    <p class="sub">
      {zone === null ? `Whole device, ${kindLabel(device.descriptor.kind).toLowerCase()}` : `Zone ${name}`}, {ledCount(targetLeds(device, zone))}
    </p>
  </div>
  <div class="actions">
    <button
      type="button"
      class="primary"
      disabled={blocked !== null || !effects.checked?.instance || effects.working !== null}
      aria-describedby="edit-mode"
      onclick={() => effects.apply()}
    >
      {effects.working === "apply" ? "Applying…" : "Apply effect"}
    </button>
    <button
      type="button"
      disabled={stopReason !== null || effects.working !== null}
      aria-describedby={stopReason ? "stop-blocked" : undefined}
      onclick={() => effects.stop()}
    >
      {effects.working === "stop" ? "Stopping…" : "Stop effect"}
    </button>
    <p id="edit-mode" class="mode">
      <span class="badge {mode.name.toLowerCase()}">{mode.name}</span>
      <span>{mode.text}</span>
    </p>
  </div>
  {#if stopReason && blocked === null}
    <p id="stop-blocked" class="note">Stop is off. {stopReason}</p>
  {/if}
  {#if condition}
    <dl class="note output"><dt>Output</dt><dd>{condition}</dd></dl>
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
  <div class="how">
    <p class="note">Runs while PrismForge runs and is connected.</p>
    <Details summary="How effects run">
      <p>A PrismForge effect is computed by PrismForge, not by the device. It runs only while PrismForge runs and is connected.</p>
      <p>{closing} When an effect stops, PrismForge asks {server} to put back the earlier lighting.</p>
      <p>PrismForge saves nothing by itself. To keep this lighting, save it as a profile on the Profiles screen. Profiles are saved on this computer, not on the device.</p>
      <p>After PrismForge starts again, no effect runs until you apply one, or until a profile marked to apply on connect is applied.</p>
    </Details>
  </div>
</section>

<style>
  .bar {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-6) var(--space-3);
    border-bottom: 1px solid var(--line-subtle);
    background: var(--surface-0);
  }

  .bar.live {
    border-bottom: 3px solid var(--ok);
  }

  .title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 var(--space-3);
  }

  .sub,
  .note {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-2);
  }

  .mode {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin-left: var(--space-2);
    flex: 1 1 18rem;
    color: var(--text-dim);
  }

  .badge {
    flex: none;
    padding: 0 var(--space-2);
    border-radius: var(--radius-xs);
    border: 1px solid var(--line-control);
    font-size: var(--font-xs);
    font-weight: var(--weight-heading);
    color: var(--text);
  }

  .badge.live {
    border-color: var(--ok);
    color: var(--ok);
  }

  .badge.off {
    border-style: dashed;
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

  .output {
    display: flex;
    gap: var(--space-2);
    margin: 0;
  }

  .output dd {
    margin: 0;
    color: var(--text);
  }

  .how {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 var(--space-3);
  }
</style>
