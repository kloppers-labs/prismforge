<script lang="ts">
  import type { Corner } from "../../lib/api/bindings/Corner";
  import type { Geometry } from "../../lib/api/bindings/Geometry";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Dialog from "../../lib/Dialog.svelte";
  import { failureText } from "../../lib/errors";
  import type { Names } from "../../lib/names";
  import type { GroupsState } from "../effects/groups.svelte";
  import { CANVAS, NUDGE, SIZE_LIMITS, placementOf } from "./edits";
  import { COMPONENT_LED_LIMIT, ledCount, reversed, withLeds } from "./geometry";
  import NumberField from "./NumberField.svelte";
  import { firstLedText, kindText, outputLine, selectionName, wiringTexts } from "./present";
  import type { SetupState } from "./state.svelte";
  import WireForm from "./WireForm.svelte";
  import { chainOf, outputOf, sameTarget, targetOf } from "./wiring";
  import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";

  let {
    setup,
    session,
    names,
    groups,
    onidentify,
  }: {
    setup: SetupState;
    session: SessionSnapshot;
    names: Names;
    groups: GroupsState;
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
  let deleting = $state(false);
  let deleteButton: HTMLButtonElement | undefined = $state();
  let grouping = $state(false);
  let groupName = $state("");
  let groupButton: HTMLButtonElement | undefined = $state();
  let opener: HTMLElement | undefined = $state();
  let members = $derived(
    ids.reduce<OutputTarget[]>((found, id) => {
      const wired = outputOf(setup.draft, id);
      return wired && !found.some((target) => sameTarget(target, targetOf(wired))) ? [...found, targetOf(wired)] : found;
    }, []),
  );

  $effect(() => {
    const asked = setup.asked;
    if (!asked) {
      return;
    }
    setup.asked = null;
    opener = asked.from;
    if (asked.kind === "delete" && one) {
      deleting = true;
    } else if (asked.kind === "group" && members.length > 0) {
      startGroup();
    }
  });

  function startGroup() {
    groupName = "";
    groups.problem = null;
    grouping = true;
  }

  async function makeGroup(event: SubmitEvent) {
    event.preventDefault();
    if (await groups.create(groupName, members)) {
      grouping = false;
    }
  }

  function reshape(geometry: Geometry, what: string) {
    if (one) {
      setup.reshape(one, geometry, what);
    }
  }

  let nameProblem = $state<string | null>(null);

  $effect(() => {
    void one;
    nameProblem = null;
  });

  function rename(input: HTMLInputElement) {
    if (!component) {
      return;
    }
    if (input.value.trim() === "") {
      nameProblem = `Enter a name. The component keeps the name ${component.name}.`;
      input.value = component.name;
      return;
    }
    nameProblem = null;
    setup.rename(component.id, input.value);
  }

  function reverseHint(geometry: Geometry): string {
    if ("Ring" in geometry || "Perimeter" in geometry) {
      return "LED 1 stays where it is and the LEDs run the other way round.";
    }
    if ("Matrix" in geometry) {
      return "LED 1 moves to the corner of the last LED, so the LEDs run in reverse.";
    }
    return "LED 1 moves to the other end, so the LEDs run in reverse.";
  }

  function bounded(value: number): number {
    return Math.min(Math.max(Math.round(value), 1), COMPONENT_LED_LIMIT);
  }

</script>

{#snippet groupButtonSnippet()}
  <button
    type="button"
    class="small"
    data-mirror="component.group"
    bind:this={groupButton}
    disabled={members.length === 0}
    aria-describedby={members.length === 0 ? "group-none" : undefined}
    onclick={() => ((opener = groupButton), startGroup())}
  >
    Make a group from the selection…
  </button>
  {#if members.length === 0}<span id="group-none" class="visually-hidden">None of the selected components is wired.</span>{/if}
{/snippet}

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
      <button type="button" class="small" data-mirror="component.turn" onclick={() => setup.turnSelection(1)}>Turn right 15°</button>
      <button type="button" class="small" data-mirror="component.flip" onclick={() => setup.flipSelection()}>Flip each</button>
    </div>
    <p class="hint">They move by the same amount, so their spacing stays as it is. Each turns and flips round its own centre.</p>
    <div class="buttons">
      {@render groupButtonSnippet()}
      <button type="button" class="small" data-mirror="component.take-out" onclick={() => setup.takeOut()}>Take out of the {setup.view?.name} view</button>
    </div>
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
        aria-describedby={nameProblem ? "component-name-problem" : undefined}
        onchange={(event) => rename(event.currentTarget)}
      />
      {#if nameProblem}<p id="component-name-problem" class="problem-text">{nameProblem}</p>{/if}
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
        <button type="button" class="small" data-mirror="component.turn" onclick={() => setup.turnSelection(1)}>Turn right 15°</button>
        <button type="button" class="small" data-mirror="component.flip" aria-pressed={placement.flipped} aria-describedby="flip-note" onclick={() => setup.flipSelection()}>Flip</button>
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
        <button type="button" class="small" onclick={() => reshape(reversed(component!.geometry), "Reverse the direction")}>Reverse the direction</button>
        <p class="hint">{reverseHint(component.geometry)} Use it when the lights run the other way round on the hardware; Identify shows the order.</p>
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
          <button type="button" class="small" data-mirror="component.identify" onclick={() => onidentify(component!.id)}>Identify</button>
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
        <WireForm {setup} {session} {names} id={component.id} />
      {/if}
    </fieldset>

    <div class="buttons">
      {@render groupButtonSnippet()}
      <button type="button" class="small" data-mirror="component.take-out" onclick={() => setup.takeOut([component!.id])}>Take out of the {setup.view?.name} view</button>
      <button type="button" class="small danger" data-mirror="component.delete" bind:this={deleteButton} onclick={() => ((opener = deleteButton), (deleting = true))}>Delete component…</button>
    </div>
  {/if}
</section>

{#if grouping}
  <Dialog title="New group" {opener} onclose={() => (grouping = false)}>
    <form id="setup-group-form" class="form" onsubmit={makeGroup}>
      <label for="setup-group-name">Group name</label>
      <input id="setup-group-name" type="text" autocomplete="off" maxlength={groups.view?.name_limit ?? 60} bind:value={groupName} />
    </form>
    <p>The group holds the {members.length === 1 ? "output" : `${members.length} outputs`} of the selected components, as a saved group on Effects. A group only selects; it changes no lighting by itself.</p>
    {#if groups.problem}<p class="problem" role="alert">{failureText(groups.problem)}</p>{/if}
    {#snippet actions()}
      <button type="button" onclick={() => (grouping = false)}>Cancel</button>
      <button type="submit" form="setup-group-form" class="primary" disabled={groups.working}>Make group</button>
    {/snippet}
  </Dialog>
{/if}

{#if deleting && component}
  <Dialog title="Delete {component.name}?" {opener} onclose={() => (deleting = false)}>
    <p>The component leaves every view and its output. Undo brings it back into the drawing until you discard changes or close PrismForge, also after a save. Your lights stay as they are.</p>
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
  select {
    width: 100%;
    min-width: 0;
  }

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

  .problems {
    margin: 0;
    padding-left: var(--space-4);
    color: var(--warning);
    font-size: var(--font-s);
  }

  .form {
    display: grid;
    gap: var(--space-1);
  }

  .problem {
    color: var(--problem);
  }

  .problem-text {
    color: var(--warning);
    font-size: var(--font-s);
  }

  .unwired {
    color: var(--warning);
    font-size: var(--font-s);
  }
</style>
