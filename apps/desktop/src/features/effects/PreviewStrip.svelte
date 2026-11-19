<script lang="ts">
  import { onMount } from "svelte";
  import { nativeWindow } from "../../lib/window.svelte";
  import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
  import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
  import { ledCount } from "../devices/present";
  import { PreviewPlayer } from "./preview.svelte";

  let { instance, leds, limits }: { instance: EffectInstance | null; leds: number; limits: PreviewLimits } = $props();

  const player = new PreviewPlayer(() => limits);
  let shown = $derived(Math.min(Math.max(leds, limits.leds.min), limits.leds.max));

  onMount(() => {
    player.open();
    return () => player.close();
  });

  $effect(() => {
    player.show(instance, leds);
  });

  $effect(() => {
    player.setWindowShown(nativeWindow.visible);
  });
</script>

<div class="preview">
  <div class="rail true-color" aria-hidden="true">
    {#if player.frame}
      {#each player.frame as color, index (index)}
        <span class="led" style:--led="rgb({color.r} {color.g} {color.b})"></span>
      {/each}
    {:else}
      {#each { length: shown }, index (index)}
        <span class="led off"></span>
      {/each}
    {/if}
  </div>
  <p class="caption">
    Preview of the pattern on {ledCount(shown)}. It is not what the device shows.
    {#if player.reduced}Motion is reduced, so the preview shows one still frame.{/if}
  </p>
  {#if instance === null}
    <p class="caption">The preview waits for valid settings.</p>
  {:else if player.failed}
    <p class="caption">The preview could not be drawn for these settings.</p>
  {/if}
</div>

<style>
  .preview {
    display: grid;
    gap: var(--space-2);
  }

  .rail {
    display: flex;
    gap: 3px;
    padding: var(--space-3) var(--space-3);
    border-radius: var(--radius-m);
    background: linear-gradient(180deg, var(--surface-stage), var(--surface-stage-edge));
    box-shadow: inset 0 0 0 1px var(--surface-3);
  }

  .led {
    flex: 1 1 0;
    min-width: 3px;
    height: 18px;
    border-radius: 3px;
    background: var(--led);
    box-shadow:
      0 0 10px -1px var(--led),
      inset 0 0 0 1px var(--line-subtle);
  }

  .led.off {
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--surface-3);
  }

  .caption {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
