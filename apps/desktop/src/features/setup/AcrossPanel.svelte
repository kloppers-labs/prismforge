<script lang="ts">
  import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import ControlField from "../effects/controls/ControlField.svelte";
  import type { AcrossState } from "./across.svelte";
  import AcrossResults from "./AcrossResults.svelte";
  import { flowText } from "./flow";
  import NumberField from "./NumberField.svelte";
  import type { SetupState } from "./state.svelte";

  let {
    across,
    setup,
    session,
    names,
    catalog,
  }: { across: AcrossState; setup: SetupState; session: SessionSnapshot; names: Names; catalog: EffectCatalog | null } = $props();

  let viewName = $derived(setup.view?.name ?? "");
  let flow = $derived(across.flow);
  let along = $derived("Along" in flow);
  let checked = $derived(across.checked);

  function percent(value: number): number {
    return Math.min(Math.max(Math.round(value), 0), 100);
  }
</script>

<section class="across" aria-label="Effect across the setup">
  <h2>Effect across the setup</h2>
  <p class="mode">
    <span class="mark" aria-hidden="true">▦</span>
    <span><strong>Across the setup:</strong> {viewName} view, {flowText(flow)}</span>
  </p>
  <p class="hint">One picture over every component in this view, on one clock. Each LED takes its colour from where it is in the drawing. LEDs without a place are sent black.</p>

  {#if !catalog}
    <p class="hint">Loading the effect list.</p>
  {:else}
    <div class="field">
      <label for="across-effect">Effect</label>
      <select id="across-effect" value={across.effect ?? ""} onchange={(event) => across.choose(event.currentTarget.value || null)}>
        <option value="">Choose an effect</option>
        {#each catalog.effects as effect (effect.id)}
          <option value={effect.id}>{effect.label}</option>
        {/each}
      </select>
    </div>
    {#if across.definition && checked}
      <div class="controls">
        {#each across.definition.controls as control (control.id)}
          {@const field = across.fields[control.id]}
          {#if field}
            <ControlField {control} {field} problem={checked.problems[control.id] ?? null} onchange={(next) => across.setField(control.id, next)} />
          {/if}
        {/each}
      </div>
    {/if}
  {/if}

  <fieldset class="flow">
    <legend>Flow</legend>
    <label><input type="radio" name="across-flow" checked={along} onchange={() => across.setFlow({ Along: { angle: 0 } })} /> Along a direction</label>
    <label><input type="radio" name="across-flow" checked={!along} onchange={() => across.setFlow({ FromCentre: { x: 50, y: 50 } })} /> Out from a centre</label>
    {#if "Along" in flow}
      <div class="row">
        <NumberField id="across-angle" label="Angle" unit="°" value={flow.Along.angle} min={0} max={359} step={15} onchange={(value) => across.setFlow({ Along: { angle: ((Math.round(value) % 360) + 360) % 360 } })} />
      </div>
      <p class="hint">0° runs from left to right, 90° from top to bottom.</p>
    {:else}
      {@const centre = flow.FromCentre}
      <div class="row">
        <NumberField id="across-x" label="Centre across" unit="%" value={centre.x} min={0} max={100} onchange={(value) => across.setFlow({ FromCentre: { ...centre, x: percent(value) } })} />
        <NumberField id="across-y" label="Centre down" unit="%" value={centre.y} min={0} max={100} onchange={(value) => across.setFlow({ FromCentre: { ...centre, y: percent(value) } })} />
      </div>
    {/if}
  </fieldset>

  <div class="actions">
    <button
      type="button"
      class="primary"
      disabled={across.blocked !== null || !checked?.instance || across.working !== null}
      aria-describedby="across-why"
      onclick={() => across.apply()}
    >
      {across.working === "apply" ? "Applying…" : `Apply across the ${viewName} view`}
    </button>
    <button type="button" disabled={across.running.length === 0 || across.working !== null || session.output.scene.assignments.length === 0} onclick={() => across.stop()}>
      {across.working === "stop" ? "Stopping…" : `Stop across the ${viewName} view`}
    </button>
  </div>
  <p id="across-why" class="hint">
    {across.blocked ?? (across.effect === null ? "Choose an effect. The drawing shows a draft of it; your devices stay as they are until you apply it." : "Your devices stay as they are until you press Apply.")}
  </p>
  <AcrossResults {across} {session} {names} />
</section>

<style>
  .across {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
  }

  h2 {
    font-size: var(--font-m);
  }

  .mode {
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--accent);
    background: var(--surface-2);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  .mark {
    color: var(--accent);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .field {
    display: grid;
    gap: var(--space-1);
    max-width: 320px;
  }

  .field label,
  legend {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .controls {
    display: grid;
    gap: var(--space-4);
  }

  .flow {
    display: grid;
    gap: var(--space-2);
    border: 0;
    margin: 0;
    padding: 0;
  }

  legend {
    padding: 0;
    margin-bottom: var(--space-1);
    font-weight: var(--weight-strong);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    max-width: 320px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  @media (forced-colors: active) {
    .mode {
      border-left-color: Highlight;
    }
  }
</style>
