<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import type { Joining } from "./edits";
  import { COMPONENT_LED_LIMIT } from "./geometry";
  import NumberField from "./NumberField.svelte";
  import { freeSpot, shapeParts, sizeOf, SHAPES, type Shape } from "./library";
  import { leds, listText, nameOf } from "./present";
  import type { SetupState } from "./state.svelte";
  import { detected, existing, type Detected } from "./targets";

  let { setup, session, names }: { setup: SetupState; session: SessionSnapshot; names: Names } = $props();

  let entries = $derived(detected(session, names));
  let open = $state<string | null>(null);
  let shape = $state<Shape>("Strip");
  let count = $state(1);
  let name = $state("");
  let joining = $state<Joining | null>(null);
  let viewName = $derived(setup.view?.name ?? "");

  function start(entry: Detected) {
    const taken = existing(setup.draft, entry);
    const last = taken?.branches.at(-1)?.at(-1);
    const lastComponent = last ? setup.draft.components.find((component) => component.id === last) : undefined;
    open = entry.key;
    shape = entry.device.descriptor.kind === "Keyboard" ? "Matrix" : "Strip";
    count = Math.min(Math.max(lastComponent && "Ring" in lastComponent.geometry ? lastComponent.geometry.Ring.leds : entry.leds, 1), COMPONENT_LED_LIMIT);
    name = entry.choice.zoneName ?? entry.title;
    joining = null;
  }

  function place(entry: Detected) {
    const taken = existing(setup.draft, entry);
    const zone = entry.choice.zone === null ? entry.device.descriptor.zones[0] : entry.device.descriptor.zones[entry.choice.zone];
    const parts = shapeParts(shape, count, zone?.matrix ?? null);
    setup.addShape(name.trim() || entry.title, parts, { ...freeSpot(setup.draft, setup.tab), size: sizeOf(shape) }, { choice: entry.choice, joining: taken ? (joining ?? "chain") : "own" });
    open = null;
  }
</script>

<section class="detected" aria-label="Detected devices">
  <h2>Place a detected device</h2>
  <p class="hint">Each device or zone that {session.backend.origin === "Simulated" ? "the demo backend" : "OpenRGB"} reports. Placing one adds a component wired to it; nothing is sent to the device.</p>
  <ul>
    {#each entries as entry (entry.key)}
      {@const taken = existing(setup.draft, entry)}
      <li role="group" aria-label={entry.title}>
        <div class="row">
          <span class="name">{entry.title}</span>
          <span class="meta">{leds(entry.leds)}</span>
        </div>
        <p class="hint">
          {#if taken}
            In the setup: {listText(taken.branches.flat().map((id) => nameOf(setup.draft, id)))}.
          {:else}
            Not in the setup yet.
          {/if}
          {#if !entry.direct}
            It has no Direct Mode, so effects cannot run on it.
          {/if}
        </p>
        {#if open === entry.key}
          <div class="form" role="group" aria-label="Place {entry.choice.zoneName ?? entry.title}">
            <div class="field">
              <label for="place-name-{entry.key}">Name</label>
              <input id="place-name-{entry.key}" type="text" autocomplete="off" bind:value={name} />
            </div>
            <div class="field">
              <label for="place-shape-{entry.key}">Shape</label>
              <select id="place-shape-{entry.key}" bind:value={shape}>
                {#each SHAPES as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
            {#if shape !== "Point"}
              <NumberField id="place-leds-{entry.key}" label="LED count" value={count} min={1} max={COMPONENT_LED_LIMIT} onchange={(value) => (count = value)} />
            {/if}
            {#if taken}
              <fieldset class="choices">
                <legend>The zone already shows {listText(taken.branches.flat().map((id) => nameOf(setup.draft, id)))}. The new component is</legend>
                <label><input type="radio" name="join-{entry.key}" value="chain" bind:group={joining} /> Next part of the daisy chain, on the LEDs after them</label>
                <label><input type="radio" name="join-{entry.key}" value="mirror" bind:group={joining} /> Mirror: the same colours, one control shown in one more place</label>
              </fieldset>
            {/if}
            <div class="actions">
              <button type="button" class="small primary" disabled={taken !== null && joining === null} onclick={() => place(entry)}>Place</button>
              <button type="button" class="small" onclick={() => (open = null)}>Cancel</button>
            </div>
            {#if taken !== null && joining === null}
              <p class="hint">PrismForge cannot see the wiring. Say whether it mirrors or continues the chain.</p>
            {/if}
          </div>
        {:else}
          <button type="button" class="small" onclick={() => start(entry)}>Place in the {viewName} view…</button>
        {/if}
      </li>
    {/each}
  </ul>
</section>

<style>
  select {
    width: 100%;
    min-width: 0;
  }

  .detected {
    display: grid;
    gap: var(--space-2);
  }

  h2 {
    font-size: var(--font-m);
  }

  ul {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    gap: var(--space-2);
    align-content: start;
    padding: var(--space-3);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
  }

  .row {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .meta {
    white-space: nowrap;
  }

  .meta,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .form,
  .choices {
    display: grid;
    gap: var(--space-2);
  }

  .choices {
    border: 0;
    margin: 0;
    padding: 0;
  }

  .choices legend {
    font-size: var(--font-s);
    color: var(--text-dim);
    margin-bottom: var(--space-1);
  }

  .field {
    display: grid;
    gap: var(--space-1);
  }

  .field label {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    gap: var(--space-1);
  }

  li > button {
    justify-self: start;
  }
</style>
