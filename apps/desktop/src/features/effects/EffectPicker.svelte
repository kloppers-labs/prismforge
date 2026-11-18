<script lang="ts">
  import type { EffectDefinition } from "../../lib/api/bindings/EffectDefinition";

  let {
    effects,
    chosen,
    running,
    onchoose,
  }: { effects: EffectDefinition[]; chosen: string | null; running: string | null; onchoose: (id: string) => void } = $props();
</script>

<fieldset class="picker">
  <legend class="visually-hidden">Effect</legend>
  {#each effects as effect (effect.id)}
    <div class="option" class:chosen={effect.id === chosen}>
      <input
        id="effect-{effect.id}"
        type="radio"
        name="effect"
        value={effect.id}
        checked={effect.id === chosen}
        aria-describedby="effect-{effect.id}-about"
        onchange={() => onchoose(effect.id)}
      />
      <label class="name" for="effect-{effect.id}">{effect.label}</label>
      {#if effect.id === running}<span class="tag">Runs here now</span>{/if}
      <p id="effect-{effect.id}-about" class="about">{effect.description}</p>
    </div>
  {/each}
</fieldset>

<style>
  .picker {
    border: 0;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 6px;
  }

  .option {
    position: relative;
    display: grid;
    grid-template-columns: auto 1fr auto;
    column-gap: 8px;
    align-items: baseline;
    padding: 9px 12px 9px 14px;
    border: 1px solid var(--line);
    border-radius: var(--radius-s);
  }

  .option:hover {
    background: var(--ink-2);
  }

  .option.chosen {
    background: var(--ink-2);
    border-color: #4a5163;
  }

  .option.chosen::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 2px;
    background: var(--spectrum);
    background-size: 100% 600%;
  }

  .name {
    font-weight: 600;
    cursor: pointer;
  }

  .tag {
    font-size: 12px;
    color: var(--ok);
  }

  .about {
    grid-column: 2 / -1;
    color: var(--text-dim);
    font-size: 13px;
  }
</style>
