<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { kindLabel, ledCount } from "../devices/present";
  import { conditionText, editMode, nearbyEffects, outputOf, stopBlocked, targetLeds, targetName } from "./present";
  import { acrossText } from "../setup/flow";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    session,
    device,
    zone,
    names,
    blocked,
    held,
  }: {
    effects: EffectsState;
    session: SessionSnapshot;
    device: SessionDevice;
    zone: number | null;
    names: Names;
    blocked: string | null;
    held: string | null;
  } = $props();

  let target = $derived(effects.target);
  let running = $derived(effects.running);
  let label = (id: string) => effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? id;
  let name = $derived(targetName(names, device, zone));
  let nearby = $derived(
    target
      ? nearbyEffects(session, target).map((assignment) => ({
          label: label(assignment.effect.effect),
          where: assignment.target.zone === null ? null : names.zone(device, assignment.target.zone),
        }))
      : [],
  );
  let mode = $derived(
    editMode({
      blocked,
      live: effects.canAutoApply,
      chosen: effects.effect,
      running: running?.effect.effect ?? null,
      runningLabel: running ? label(running.effect.effect) : null,
      nearby,
      target: name,
    }),
  );
  let stopReason = $derived(held ?? (target ? stopBlocked(session, target) : null));
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
      data-mirror="effect.apply"
      disabled={blocked !== null || !effects.checked?.instance || effects.working !== null}
      aria-describedby="edit-mode"
      onclick={() => effects.apply()}
    >
      {effects.working === "apply" ? "Applying…" : "Apply effect"}
    </button>
    <button
      type="button"
      data-mirror="target.stop"
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
  {#if typeof effects.mode === "object"}
    <p class="across"><span aria-hidden="true">▦</span> {acrossText(effects.mode.AcrossSetup.view, effects.mode.AcrossSetup.flow)}. Changes keep this mode.</p>
  {/if}
  {#if condition}
    <dl class="output"><dt>Output</dt><dd>{condition}</dd></dl>
  {/if}
</section>

<style>
  .across {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .bar {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-6);
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

  .sub {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
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

  .output {
    display: flex;
    gap: var(--space-2);
    margin: 0;
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .output dd {
    margin: 0;
    color: var(--text);
  }
</style>
