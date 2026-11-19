<script lang="ts">
  import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
  import type { EffectDefinition } from "../../lib/api/bindings/EffectDefinition";
  import type { Menu } from "../../lib/shell/menu.svelte";
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import ControlField from "./controls/ControlField.svelte";
  import EffectPicker from "./EffectPicker.svelte";
  import { targetLeds } from "./present";
  import PreviewPanel from "./PreviewPanel.svelte";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    catalog,
    device,
    zone,
    locked,
    menu,
  }: {
    effects: EffectsState;
    catalog: EffectCatalog;
    device: SessionDevice;
    zone: number | null;
    locked: boolean;
    menu: (effect: EffectDefinition) => Menu | null;
  } = $props();

  let checked = $derived(effects.checked);
</script>

<section class="editor" aria-labelledby="effect-title">
  <h3 id="effect-title">Effect</h3>
  <EffectPicker
    effects={catalog.effects}
    chosen={effects.effect}
    running={effects.running?.effect.effect ?? null}
    disabled={locked}
    onchoose={(id) => effects.choose(id)}
    {menu}
  />

  {#if effects.definition && checked}
    <div class="controls">
      <h3>Settings for {effects.definition.label}</h3>
      {#each effects.definition.controls as control (control.id)}
        {@const field = effects.fields[control.id]}
        {#if field}
          <ControlField
            {control}
            {field}
            problem={checked.problems[control.id] ?? effects.rustProblems[control.id] ?? null}
            onchange={(next) => effects.setField(control.id, next)}
          />
        {/if}
      {/each}
    </div>
    <PreviewPanel instance={checked.instance} leds={targetLeds(device, zone)} limits={catalog.preview} />
  {/if}
</section>

<style>
  .editor {
    display: grid;
    gap: var(--space-4);
  }

  h3 {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .controls {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
  }
</style>
