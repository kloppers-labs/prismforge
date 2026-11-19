<script lang="ts">
  import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { serverName } from "../connection/wording";
  import ControlField from "./controls/ControlField.svelte";
  import EffectPicker from "./EffectPicker.svelte";
  import FailureNotice from "./FailureNotice.svelte";
  import { applyBlocked, stopBlocked, stopText, targetLeds } from "./present";
  import PreviewPanel from "./PreviewPanel.svelte";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    catalog,
    session,
    device,
    zone,
    held,
  }: {
    effects: EffectsState;
    catalog: EffectCatalog;
    session: SessionSnapshot;
    device: SessionDevice;
    zone: number | null;
    held: string | null;
  } = $props();

  let target = $derived(effects.target);
  let running = $derived(effects.running);
  let runningLabel = $derived(catalog.effects.find((effect) => effect.id === running?.effect.effect)?.label ?? null);
  let blocked = $derived(held ?? applyBlocked(session, device, zone));
  let stopReason = $derived(held ?? (target ? stopBlocked(session, target) : null));
  let checked = $derived(effects.checked);
  let assigned = $derived(running !== null && running.effect.effect === effects.effect);
  let server = $derived(serverName(session));
  let feedback = $derived(effects.feedback);
</script>

<section class="editor" aria-labelledby="effect-title">
  <h3 id="effect-title">Effect</h3>
  <EffectPicker effects={catalog.effects} chosen={effects.effect} running={running?.effect.effect ?? null} onchoose={(id) => effects.choose(id)} />

  {#if effects.definition && checked}
    <div class="controls">
      <h4>Settings for {effects.definition.label}</h4>
      {#each effects.definition.controls as control (control.id)}
        {@const field = effects.fields[control.id]}
        {#if field}
          <ControlField
            {control}
            {field}
            problem={checked.problems[control.id] ?? effects.rustProblems[control.id] ?? null}
            onchange={(next) => effects.setField(control.id, next)}
          />
        {/if}
      {/each}
    </div>
    <PreviewPanel instance={checked.instance} leds={targetLeds(device, zone)} limits={catalog.preview} />
  {/if}

  <section class="run" aria-labelledby="run-title">
    <h4 id="run-title">Run on this target</h4>
    <p class="lifecycle">
      A PrismForge effect is computed by PrismForge, not by the device. It runs only while PrismForge runs and is
      connected. Closing the window quits PrismForge and stops its effects. PrismForge then asks {server} to put back the
      earlier lighting. PrismForge saves nothing by itself. To keep this lighting, save it as a profile on the Profiles screen.
      Profiles are saved on this computer, not on the device. After PrismForge starts again, no effect runs until you apply
      it or a profile again.
    </p>
    {#if effects.canAutoApply}
      <p class="hint">This is the effect on this target. Changes apply as you make them, and the animation keeps its timing.</p>
    {:else if assigned && blocked === null}
      <p class="hint">This is the effect on this target. Press Apply effect to send your changes.</p>
    {:else if runningLabel && effects.effect && !assigned && blocked === null}
      <p class="hint">{runningLabel} is the effect on this target. Apply effect replaces it.</p>
    {/if}
    <div class="buttons">
      <button
        type="button"
        class="primary"
        disabled={blocked !== null || !checked?.instance || effects.working !== null}
        aria-describedby={blocked ? "apply-blocked" : undefined}
        onclick={() => effects.apply()}
      >
        {effects.working === "apply" ? "Applying" : "Apply effect"}
      </button>
      <button
        type="button"
        disabled={stopReason !== null || effects.working !== null}
        aria-describedby={stopReason ? "stop-blocked" : undefined}
        onclick={() => effects.stop()}
      >
        {effects.working === "stop" ? "Stopping" : "Stop effect"}
      </button>
    </div>
    {#if blocked}
      <p id="apply-blocked" class="blocked">Effects are off. {blocked}</p>
    {:else if !effects.effect}
      <p class="blocked">Choose an effect above.</p>
    {/if}
    {#if stopReason}
      <p id="stop-blocked" class="blocked">Stop is off. {stopReason}</p>
    {/if}

    {#if feedback?.kind === "failed"}
      <FailureNotice error={feedback.error} />
    {/if}
    <div class="result" role="status" aria-label="Effect result">
      {#if feedback?.kind === "applied"}
        <p class="outcome">PrismForge accepted {feedback.label} for {feedback.targetLabel}. The Output line above shows whether it runs.</p>
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
  </section>
</section>

<style>
  .editor {
    display: grid;
    gap: 16px;
  }

  h4 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-dim);
  }

  .controls {
    display: grid;
    gap: 18px;
    padding: 16px 18px;
    border: 1px solid var(--line);
    border-radius: var(--radius-m);
    background: var(--ink-1);
  }

  .run {
    display: grid;
    gap: 10px;
  }

  .lifecycle,
  .hint,
  .blocked {
    color: var(--text-dim);
    max-width: 72ch;
  }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .result:empty {
    display: none;
  }

  .outcome {
    display: grid;
    gap: 4px;
    padding: 10px 14px;
    border-left: 3px solid var(--ok);
    background: var(--ink-0);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }

  .outcome.unsettled {
    border-left-color: var(--warning);
  }

  .headline {
    color: var(--text);
    font-weight: 600;
  }
</style>
