<script lang="ts">
  import type { Corner } from "../../lib/api/bindings/Corner";
  import type { Geometry } from "../../lib/api/bindings/Geometry";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Dialog from "../../lib/Dialog.svelte";
  import type { Names } from "../../lib/names";
  import { CANVAS, NUDGE, SIZE_LIMITS, placementOf, type Joining } from "./edits";
  import { COMPONENT_LED_LIMIT, ledCount, reversed, withLeds } from "./geometry";
  import NumberField from "./NumberField.svelte";
  import { firstLedText, kindText, nameOf, outputLine, selectionName, wiringTexts } from "./present";
  import type { SetupState } from "./state.svelte";
  import { detected, existing } from "./targets";
  import { chainOf, outputOf } from "./wiring";

  let {
    setup,
    session,
    names,
    onidentify,
  }: {
    setup: SetupState;
    session: SessionSnapshot;
    names: Names;
    onidentify: (id: string) => void;
  } = $props();

  const CORNERS: [Corner, string][] = [
    ["TopLeft", "Top left"],
    ["TopRight", "Top right"],
    ["BottomRight", "Bottom right"],
    ["BottomLeft", "Bottom left"],
  ];

  let ids = $derived(setup.selection);
  let one = $derived(ids.length === 1 ? ids[0]! : null);
  let component = $derived(one ? (setup.draft.components.find((each) => each.id === one) ?? null) : null);
  let placement = $derived(one ? placementOf(setup.draft, setup.tab, one) : null);
  let output = $derived(one ? outputOf(setup.draft, one) : null);
  let chain = $derived(one ? chainOf(setup.draft, one) : null);
  let issues = $derived(setup.issues.filter((issue) => issue.component === one || (issue.component === null && output !== null && issue.output === output.id)));
  let title = $derived(ids.length === 0 ? "Edit the selection" : `Edit ${selectionName(setup.draft, ids)}`);
  let choices = $derived(detected(session, names));
  let wireTo = $state("");
  let joining = $state<Joining | null>(null);
  let chosen = $derived(choices.find((entry) => entry.key === wireTo) ?? null);
  let taken = $derived(chosen ? existing(setup.draft, chosen) : null);
  let deleting = $state(false);
  let deleteButton: HTMLButtonElement | undefined = $state();

  function reshape(geometry: Geometry, what: string) {
    if (one) {
      setup.reshape(one, geometry, what);
    }
  }

  function listNames(ids: string[]): string {
    return ids.length === 1 ? nameOf(setup.draft, ids[0]!) : `${ids.length} components`;
  }

  function bounded(value: number): number {
    return Math.min(Math.max(Math.round(value), 1), COMPONENT_LED_LIMIT);
  }

  function wire() {
    if (!one || !chosen) {
      return;
    }
    setup.wire(one, chosen.choice, taken ? (joining ?? "chain") : "own");
    wireTo = "";
    joining = null;
  }
</script>

<section class="inspector" aria-label={title}>
  {#if ids.length === 0}
    <p class="hint">Select a component in the drawing or the list to move, turn, flip, rename or wire it.</p>
  {:else if ids.length > 1}
    <h2 class="heading">{ids.length} components selected</h2>
    <div class="buttons" role="group" aria-label="Move and turn together">
      <button type="button" class="small" onclick={() => setup.moveSelection(-NUDGE, 0)}>Move left</button>
      <button type="button" class="small" onclick={() => setup.moveSelection(NUDGE, 0)}>Move right</button>
      <button type="button" class="small" onclick={() => setup.moveSelection(0, -NUDGE)}>Move up</button>
      <button type="button" class="small" onclick={() => setup.moveSelection(0, NUDGE)}>Move down</button>
      <button type="button" class="small" onclick={() => setup.turnSelection(-1)}>Turn left 15°</button>
      <button type="button" class="small" onclick={() => setup.turnSelection(1)}>Turn right 15°</button>
      <button type="button" class="small" onclick={() => setup.flipSelection()}>Flip each</button>
    </div>
    <p class="hint">They move by the same amount, so their spacing stays as it is. Each turns and flips round its own centre.</p>
    <button type="button" class="small" onclick={() => setup.takeOut()}>Take out of the {setup.view?.name} view</button>
  {:else if component && placement}
    <h2 class="heading">{component.name}</h2>
    <p class="kind">{kindText(component.geometry)}</p>

    <div class="block">
      <label for="component-name">Name</label>
      <input
        id="component-name"
        type="text"
        autocomplete="off"
        value={component.name}
        onchange={(event) => event.currentTarget.value.trim() !== "" && setup.rename(component!.id, event.currentTarget.value)}
      />
    </div>

    <fieldset class="block">
      <legend>Place in the {setup.view?.name} view</legend>
      <div class="grid">
        <NumberField id="place-x" label="X" value={placement.x} min={0} max={CANVAS.width} onchange={(value) => setup.setPlace(component!.id, "x", value)} />
        <NumberField id="place-y" label="Y" value={placement.y} min={0} max={CANVAS.height} onchange={(value) => setup.setPlace(component!.id, "y", value)} />
        <NumberField id="place-size" label="Size" value={placement.size} min={SIZE_LIMITS.min} max={SIZE_LIMITS.max} onchange={(value) => setup.setPlace(component!.id, "size", value)} />
        <NumberField id="place-turn" label="Turn" unit="°" value={placement.rotation} min={0} max={359} step={15} onchange={(value) => setup.setPlace(component!.id, "rotation", value)} />
      </div>
      <div class="buttons">
        <button type="button" class="small" onclick={() => setup.moveSelection(-NUDGE, 0)}>Move left</button>
        <button type="button" class="small" onclick={() => setup.moveSelection(NUDGE, 0)}>Move right</button>
        <button type="button" class="small" onclick={() => setup.moveSelection(0, -NUDGE)}>Move up</button>
        <button type="button" class="small" onclick={() => setup.moveSelection(0, NUDGE)}>Move down</button>
        <button type="button" class="small" onclick={() => setup.turnSelection(-1)}>Turn left 15°</button>
        <button type="button" class="small" onclick={() => setup.turnSelection(1)}>Turn right 15°</button>
        <button type="button" class="small" aria-pressed={placement.flipped} aria-describedby="flip-note" onclick={() => setup.flipSelection()}>Flip</button>
      </div>
      <p id="flip-note" class="hint">Flip mirrors the drawing left to right. The order of the LEDs on the wire stays the same.</p>
    </fieldset>

    <fieldset class="block">
      <legend>LEDs</legend>
      {#if "Strip" in component.geometry || "Ring" in component.geometry || "Line" in component.geometry}
        <NumberField
          id="shape-leds"
          label="LED count"
          value={ledCount(component.geometry)}
          min={1}
          max={COMPONENT_LED_LIMIT}
          describedby={issues.length > 0 ? "shape-problems" : undefined}
          onchange={(value) => reshape(withLeds(component!.geometry, bounded(value)), "Change the LED count")}
        />
      {:else if "Matrix" in component.geometry}
        {@const matrix = component.geometry.Matrix}
        <div class="grid">
          <NumberField id="shape-columns" label="Columns" value={matrix.columns} min={1} max={COMPONENT_LED_LIMIT} onchange={(value) => reshape({ Matrix: { ...matrix, columns: bounded(value) } }, "Change the columns")} />
          <NumberField id="shape-rows" label="Rows" value={matrix.rows} min={1} max={COMPONENT_LED_LIMIT} onchange={(value) => reshape({ Matrix: { ...matrix, rows: bounded(value) } }, "Change the rows")} />
        </div>
      {:else if "Perimeter" in component.geometry}
        {@const edge = component.geometry.Perimeter}
        <div class="grid">
          {#each ["top", "right", "bottom", "left"] as const as side (side)}
            <NumberField
              id="shape-{side}"
              label="LEDs {side === 'top' || side === 'bottom' ? 'along the' : 'on the'} {side}"
              value={edge[side]}
              min={0}
              max={COMPONENT_LED_LIMIT}
              onchange={(value) => reshape({ Perimeter: { ...edge, [side]: Math.max(0, Math.round(value)) } }, `Change the LEDs on the ${side}`)}
            />
          {/each}
        </div>
      {/if}
      {#if issues.length > 0}
        <ul id="shape-problems" class="problems">
          {#each issues as issue (issue.text)}
            <li>{issue.text}</li>
          {/each}
        </ul>
      {/if}
      <p class="hint">{firstLedText(component.geometry, placement)}.</p>
      {#if "Ring" in component.geometry}
        {@const ring = component.geometry.Ring}
        <NumberField id="shape-angle" label="LED 1 angle from the top" unit="°" value={ring.first_angle} min={0} max={359} onchange={(value) => reshape({ Ring: { ...ring, first_angle: ((Math.round(value) % 360) + 360) % 360 } }, "Move LED 1")} />
      {:else if "Matrix" in component.geometry || "Perimeter" in component.geometry}
        {@const corner = "Matrix" in component.geometry ? component.geometry.Matrix.first : component.geometry.Perimeter.first}
        <div class="field">
          <label for="shape-corner">Corner of LED 1</label>
          <select
            id="shape-corner"
            value={corner}
            onchange={(event) => {
              const first = event.currentTarget.value as Corner;
              const geometry = component!.geometry;
              reshape("Matrix" in geometry ? { Matrix: { ...geometry.Matrix, first } } : "Perimeter" in geometry ? { Perimeter: { ...geometry.Perimeter, first } } : geometry, "Move LED 1");
            }}
          >
            {#each CORNERS as [value, text] (value)}
              <option {value}>{text}</option>
            {/each}
          </select>
        </div>
      {/if}
      {#if !("Point" in component.geometry)}
        <button type="button" class="small" onclick={() => reshape(reversed(component!.geometry), "Reverse the LED order")}>Reverse the LED order</button>
        <p class="hint">Use this when the lights run the other way round on the hardware. Identify shows the order.</p>
      {/if}
    </fieldset>

    <fieldset class="block">
      <legend>Wiring</legend>
      {#if output}
        <p>{outputLine(output, names, session)}</p>
        {#each wiringTexts(setup.draft, component.id, names, session) as text (text)}
          <p class="hint">{text}</p>
        {/each}
        <div class="buttons">
          {#if chain && chain.length > 1}
            <button type="button" class="small" disabled={chain.index === 0} onclick={() => setup.moveInChain(component!.id, -1)}>Earlier in the chain</button>
            <button type="button" class="small" disabled={chain.index === chain.length - 1} onclick={() => setup.moveInChain(component!.id, 1)}>Later in the chain</button>
          {/if}
          <button type="button" class="small" onclick={() => onidentify(component!.id)}>Identify</button>
          <button type="button" class="small" onclick={() => setup.unwire(component!.id)}>Unwire</button>
        </div>
        <NumberField
          id="output-first"
          label="Starts at LED"
          value={output.first_led + 1}
          min={1}
          max={COMPONENT_LED_LIMIT}
          describedby="output-first-note"
          onchange={(value) => setup.setFirstLed(output!.id, Math.max(0, Math.round(value) - 1))}
        />
        <p id="output-first-note" class="hint">The first LED of the zone that this output uses. Most outputs start at LED 1.</p>
      {:else}
        <p class="unwired">Not wired. It shows no colour and gets nothing until you choose its output.</p>
        <div class="field">
          <label for="wire-to">Output</label>
          <select id="wire-to" bind:value={wireTo}>
            <option value="">Choose a detected device or zone</option>
            {#each choices as entry (entry.key)}
              <option value={entry.key}>{entry.title}</option>
            {/each}
          </select>
        </div>
        {#if chosen && taken}
          <fieldset class="choices">
            <legend>{chosen.title} already shows {listNames(taken.branches.flat())}. This component is</legend>
            <label><input type="radio" name="joining" value="mirror" bind:group={joining} /> Mirror: the same colours, one control</label>
            <label><input type="radio" name="joining" value="chain" bind:group={joining} /> Next part of the daisy chain, on the following LEDs</label>
          </fieldset>
        {/if}
        <button type="button" class="small" disabled={!chosen || (taken !== null && joining === null)} onclick={wire}>Wire</button>
      {/if}
    </fieldset>

    <div class="buttons">
      <button type="button" class="small" onclick={() => setup.takeOut([component!.id])}>Take out of the {setup.view?.name} view</button>
      <button type="button" class="small danger" bind:this={deleteButton} onclick={() => (deleting = true)}>Delete component…</button>
    </div>
  {/if}
</section>

{#if deleting && component}
  <Dialog title="Delete {component.name}?" opener={deleteButton} onclose={() => (deleting = false)}>
    <p>The component leaves every view and its output. Undo brings it back until you save. Your lights stay as they are.</p>
    {#snippet actions()}
      <button type="button" onclick={() => (deleting = false)}>Keep component</button>
      <button
        type="button"
        class="danger"
        onclick={() => {
          setup.remove(component!.id);
          deleting = false;
        }}>Delete component</button
      >
    {/snippet}
  </Dialog>
{/if}

<style>
  .inspector {
    display: grid;
    gap: var(--space-3);
    align-content: start;
  }

  .heading {
    font-size: var(--font-l);
  }

  .kind,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .block {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }

  legend {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
    padding: 0;
    margin-bottom: var(--space-1);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
    gap: var(--space-2);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .field {
    display: grid;
    gap: var(--space-1);
  }

  .field label {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .choices {
    display: grid;
    gap: var(--space-1);
    border: 0;
    padding: 0;
    margin: 0;
  }

  .problems {
    margin: 0;
    padding-left: var(--space-4);
    color: var(--warning);
    font-size: var(--font-s);
  }

  .unwired {
    color: var(--warning);
    font-size: var(--font-s);
  }
</style>
