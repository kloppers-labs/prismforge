<script lang="ts">
  import type { Saved } from "../../lib/api/bindings/Saved";
  import { changeText, deviceLabel, type Labels } from "./present";

  let { saved, labels }: { saved: Saved; labels: Labels } = $props();
</script>

<ul class="changes">
  {#each saved.changes as change, index (index)}
    <li><strong>{deviceLabel(labels, change.device, change.label)}</strong>: {changeText(labels, change.device, change.change)}.</li>
  {/each}
  {#if saved.frame_rate}
    <li>Frame rate: from {saved.frame_rate.was} to {saved.frame_rate.now} frames per second.</li>
  {/if}
</ul>

<style>
  .changes {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 4px;
  }
</style>
