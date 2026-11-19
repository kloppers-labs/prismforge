<script lang="ts">
  import type { OutlineKind } from "../../lib/api/bindings/OutlineKind";
  import { OUTLINES } from "./library";
  import type { SetupState } from "./state.svelte";

  let { setup }: { setup: SetupState } = $props();

  let kind = $state<OutlineKind>("Case");
</script>

<section class="outlines" aria-label="Outlines without LEDs">
  <h2>Add an outline</h2>
  <p class="hint">A case, a desk, a monitor or a PC drawn as a plain outline, to place the lit components by. It has no LEDs; nothing is wired to it and no effect uses it.</p>
  <div class="row">
    <div class="field">
      <label for="outline-new-kind">Outline</label>
      <select id="outline-new-kind" bind:value={kind}>
        {#each OUTLINES as option (option.kind)}
          <option value={option.kind}>{option.name}</option>
        {/each}
      </select>
    </div>
    <button type="button" class="small" onclick={() => setup.addOutline(kind)}>Add the outline</button>
  </div>
</section>

<style>
  .outlines,
  .field {
    display: grid;
    gap: var(--space-2);
  }

  .field {
    gap: var(--space-1);
  }

  h2 {
    font-size: var(--font-m);
  }

  .hint,
  label {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: var(--space-2);
  }
</style>
