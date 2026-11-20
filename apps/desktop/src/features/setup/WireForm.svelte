<script lang="ts">
  import type { ComponentId } from "../../lib/api/bindings/ComponentId";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import type { Joining } from "./edits";
  import { listText, nameOf } from "./present";
  import type { SetupState } from "./state.svelte";
  import { detected, existing } from "./targets";

  let { setup, session, names, id }: { setup: SetupState; session: SessionSnapshot; names: Names; id: ComponentId } = $props();

  let choices = $derived(detected(session, names));
  let wireTo = $state("");
  let joining = $state<Joining | null>(null);
  let chosen = $derived(choices.find((entry) => entry.key === wireTo) ?? null);
  let taken = $derived(chosen ? existing(setup.draft, chosen) : null);
  let others = $derived(taken ? listText(taken.branches.flat().map((each) => nameOf(setup.draft, each))) : "");

  function wire() {
    if (!chosen) {
      return;
    }
    setup.wire(id, chosen.choice, taken ? (joining ?? "chain") : "own");
    wireTo = "";
    joining = null;
  }
</script>

<div class="wire">
  <div class="field">
    <label for="wire-to-{id}">Output</label>
    <select id="wire-to-{id}" bind:value={wireTo} onchange={() => (joining = null)}>
      <option value="">Choose a detected device or zone</option>
      {#each choices as entry (entry.key)}
        <option value={entry.key}>{entry.title}</option>
      {/each}
    </select>
  </div>
  {#if chosen && taken}
    <fieldset class="choices">
      <legend>{chosen.title} already shows {others}. This component is</legend>
      <label><input type="radio" name="joining-{id}" value="mirror" bind:group={joining} /> Mirror: the same colours as {others}, one control</label>
      <label><input type="radio" name="joining-{id}" value="chain" bind:group={joining} /> Next part of the daisy chain, on the LEDs after them</label>
    </fieldset>
    {#if joining === null}
      <p class="hint">PrismForge cannot see the wiring. Say whether it mirrors or continues the chain.</p>
    {/if}
  {/if}
  <button type="button" class="small" disabled={!chosen || (taken !== null && joining === null)} onclick={wire}>Wire</button>
</div>

<style>
  select {
    width: 100%;
    min-width: 0;
  }

  .wire,
  .field,
  .choices {
    display: grid;
    gap: var(--space-2);
  }

  .field {
    gap: var(--space-1);
  }

  .field label,
  .hint,
  legend {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .choices {
    border: 0;
    margin: 0;
    padding: 0;
  }

  legend {
    padding: 0;
    margin-bottom: var(--space-1);
  }

  button {
    justify-self: start;
  }
</style>
