<script lang="ts">
  import type { EffectDefinition } from "../../lib/api/bindings/EffectDefinition";
  import { contextMenu, type Menu } from "../../lib/shell/menu.svelte";

  let {
    effects,
    chosen,
    running,
    disabled,
    onchoose,
    menu,
  }: {
    effects: EffectDefinition[];
    chosen: string | null;
    running: string | null;
    disabled: boolean;
    onchoose: (id: string) => void;
    menu: (effect: EffectDefinition) => Menu | null;
  } = $props();
</script>

<fieldset class="picker" aria-labelledby="effect-title" {disabled}>
  {#each effects as effect (effect.id)}
    <label class="pick card" class:off={disabled} use:contextMenu={() => menu(effect)}>
      <span class="head">
        <input
          type="radio"
          name="effect"
          value={effect.id}
          data-mirror="effect.choose"
          checked={effect.id === chosen}
          aria-labelledby="effect-{effect.id}-name"
          aria-describedby="effect-{effect.id}-about"
          onchange={() => onchoose(effect.id)}
        />
        <span id="effect-{effect.id}-name" class="pick-name">{effect.label}</span>
        {#if effect.id === running}<span class="tag good">Runs now</span>{/if}
      </span>
      <span id="effect-{effect.id}-about" class="pick-meta">{effect.description}</span>
    </label>
  {/each}
</fieldset>

<style>
  .picker {
    border: 0;
    margin: 0;
    padding: 0;
    min-width: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 13rem), 1fr));
    gap: var(--space-2);
  }

  .card {
    align-content: start;
    border-color: var(--line-subtle);
    cursor: pointer;
  }

  .card.off {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  input {
    margin: 0;
    accent-color: var(--accent);
  }

  .card:has(input:focus-visible) {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }
</style>
