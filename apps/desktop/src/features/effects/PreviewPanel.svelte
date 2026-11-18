<script lang="ts">
  import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
  import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
  import PreviewStrip from "./PreviewStrip.svelte";

  let { instance, leds, limits }: { instance: EffectInstance | null; leds: number; limits: PreviewLimits } = $props();

  let open = $state(true);
</script>

<div class="panel">
  <div class="head">
    <h4 id="preview-title">Preview</h4>
    <button type="button" class="quiet" aria-expanded={open} aria-controls="preview-body" onclick={() => (open = !open)}>
      {open ? "Hide preview" : "Show preview"}
    </button>
  </div>
  <div id="preview-body">
    {#if open}
      <PreviewStrip {instance} {leds} {limits} />
    {/if}
  </div>
</div>

<style>
  .panel {
    display: grid;
    gap: 8px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h4 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-dim);
  }

  .quiet {
    padding: 3px 10px;
    font-size: 13px;
  }
</style>
