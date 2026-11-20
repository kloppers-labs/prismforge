<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { kindText } from "./present";
  import type { SetupState } from "./state.svelte";
  import { outputOf } from "./wiring";
  import WireForm from "./WireForm.svelte";

  let { setup, session, names }: { setup: SetupState; session: SessionSnapshot; names: Names } = $props();

  let unwired = $derived(setup.draft.components.filter((component) => setup.placed.includes(component.id) && outputOf(setup.draft, component.id) === null));
</script>

{#if unwired.length > 0}
  <section class="unwired" aria-label="Not wired yet">
    <h2>Not wired yet</h2>
    <p class="hint">These components show no colour and get nothing. Choose the detected device or zone each one is on. PrismForge wires nothing by itself.</p>
    <ul>
      {#each unwired as component (component.id)}
        <li role="group" aria-label={component.name}>
          <p class="name">{component.name} <span class="meta">{kindText(component.geometry)}</span></p>
          <WireForm {setup} {session} {names} id={component.id} />
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .unwired {
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
    border: 1px dashed var(--warning-line);
    border-radius: var(--radius-m);
    background: var(--surface-1);
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .meta,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
    font-weight: var(--weight-regular);
  }
</style>
