<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { outputLines, outputName } from "./present";
  import type { SetupState } from "./state.svelte";

  let { setup, session, names }: { setup: SetupState; session: SessionSnapshot; names: Names } = $props();

  let outputs = $derived(setup.draft.outputs);
</script>

<section class="outputs" aria-labelledby="outputs-title">
  <h2 id="outputs-title">Outputs</h2>
  <p class="hint">Each output is one control: a device or a zone that PrismForge writes as one. Counts are for the {setup.view?.name} view.</p>
  {#if outputs.length === 0}
    <p class="hint">No component is wired yet.</p>
  {:else}
    <ul aria-label="Outputs in the setup">
      {#each outputs as output (output.id)}
        {@const name = outputName(output, names, session)}
        <li>
          <p class="name">{name.zone === null ? name.device : `${name.zone} on ${name.device}`}</p>
          {#each outputLines(setup.draft, output, setup.tab, session) as line (line)}
            <p class="line">{line}</p>
          {/each}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .outputs {
    display: grid;
    gap: var(--space-2);
  }

  h2 {
    font-size: var(--font-m);
  }

  ul {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-subtle);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .line,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
