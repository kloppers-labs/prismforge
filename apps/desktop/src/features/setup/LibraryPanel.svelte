<script lang="ts">
  import { placementOf } from "./edits";
  import { kindText } from "./present";
  import type { SetupState } from "./state.svelte";
  import { TEMPLATES } from "./templates";

  let { setup }: { setup: SetupState } = $props();

  let viewName = $derived(setup.view?.name ?? "");
  let templates = $derived(TEMPLATES.filter((template) => template.view === setup.tab));
  let elsewhere = $derived(setup.draft.components.filter((component) => !setup.placed.includes(component.id)));

  function placeHere(id: string) {
    const other = setup.draft.views.map((view) => placementOf(setup.draft, view.id, id)).find((placement) => placement !== null);
    setup.placeHere(id, other ? { x: other.x, y: other.y, size: other.size, rotation: other.rotation } : { x: 500, y: 350 });
  }
</script>

<section class="library" aria-labelledby="library-title">
  <h2 id="library-title">Start from a template</h2>
  {#each templates as template (template.id)}
    <div class="template">
      <p class="hint">
        Proposes {template.slots.length} generic components for a {template.label.toLowerCase()}: {template.slots.map((slot) => slot.name.toLowerCase()).join(", ")}. It also draws {template.outlines.map((outline) => outline.name.toLowerCase()).join(", ")} as {template.outlines.length === 1 ? "an outline" : "outlines"} without LEDs. It wires none of them; you choose the output of each, and you can move, change or delete any of them.
        {setup.placed.length > 0 ? `They are added beside what is in the ${viewName} view.` : ""}
      </p>
      <button type="button" class="small" onclick={() => setup.addTemplate(template)}>Add the {template.label} template</button>
    </div>
  {/each}
</section>

{#if elsewhere.length > 0}
  <section class="library" aria-label="Not in this view">
    <h2>In the setup, not in the {viewName} view</h2>
    <p class="hint">A component can be in both views. Its wiring is the same in each.</p>
    <ul>
      {#each elsewhere as component (component.id)}
        <li role="group" aria-label={component.name}>
          <span class="name">{component.name}</span>
          <span class="hint">{kindText(component.geometry)}</span>
          <button type="button" class="small" onclick={() => placeHere(component.id)}>Place in the {viewName} view</button>
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .library,
  .template {
    display: grid;
    gap: var(--space-2);
  }

  h2 {
    font-size: var(--font-m);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .template button {
    justify-self: start;
  }

  ul {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .name {
    font-weight: var(--weight-strong);
  }
</style>
