<script lang="ts">
  import type { Rgb } from "../../lib/api/bindings/Rgb";

  let { colors }: { colors: Rgb[] } = $props();

  let shown = $derived(colors.slice(0, 120));
</script>

<div class="strip" aria-hidden="true">
  {#each shown as color, index (index)}
    <span style:background-color="rgb({color.r} {color.g} {color.b})"></span>
  {/each}
</div>

<style>
  .strip {
    display: flex;
    gap: 2px;
    height: 14px;
  }

  span {
    flex: 1 1 0;
    min-width: 2px;
    max-width: 18px;
    border-radius: 2px;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.08);
  }
</style>
