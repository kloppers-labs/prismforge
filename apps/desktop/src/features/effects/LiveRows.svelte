<script lang="ts">
  import type { LiveOutput } from "../../lib/api/bindings/LiveOutput";
  import type { LiveZone } from "../../lib/api/bindings/LiveZone";
  import type { StageRow } from "./present";

  let { rows, output }: { rows: StageRow[]; output: LiveOutput | null } = $props();

  function zonesOf(row: StageRow): LiveZone[] {
    const device = output?.devices.find((each) => each.device === row.target.device);
    return device?.zones.filter((zone) => row.target.zone === null || zone.zone === row.target.zone) ?? [];
  }

  function columns(zone: LiveZone): number {
    return zone.matrix && zone.matrix.width > 0 ? zone.matrix.width : 0;
  }
</script>

{#each rows as row (row.key)}
  {@const zones = zonesOf(row)}
  <section class="target" aria-label={row.label}>
    <p class="label">{row.label}</p>
    {#if zones.length === 0}
      <p class="none">{output === null ? "Waiting for the first frame." : "PrismForge sends nothing to this target now."}</p>
    {/if}
    {#each zones as zone (zone.zone)}
      {@const width = columns(zone)}
      <div class="zone">
        {#if zones.length > 1 || row.target.zone !== null}<p class="zone-name">{zone.name}</p>{/if}
        <div
          class="leds true-color"
          class:grid={width > 0}
          style:--columns={width > 0 ? width : undefined}
          role="img"
          aria-label="{zone.name}: {width > 0 ? `a grid of ${width} by ${zone.matrix?.height ?? 0}` : `a row of ${zone.colors.length} LEDs`}, as PrismForge last sent them"
        >
          {#each zone.colors as color, index (index)}
            <span class="led" style:--led="rgb({color.r} {color.g} {color.b})"></span>
          {/each}
        </div>
      </div>
    {/each}
  </section>
{/each}

<style>
  .target,
  .zone {
    display: grid;
    gap: var(--space-1);
  }

  .label,
  .zone-name,
  .none {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .leds {
    display: flex;
    gap: 3px;
    padding: var(--space-3);
    border-radius: var(--radius-m);
    background: linear-gradient(180deg, var(--surface-stage), var(--surface-stage-edge));
    box-shadow: inset 0 0 0 1px var(--surface-3);
  }

  .leds.grid {
    display: grid;
    grid-template-columns: repeat(var(--columns), minmax(3px, 1fr));
  }

  .led {
    flex: 1 1 0;
    min-width: 3px;
    height: 18px;
    border-radius: 3px;
    background: var(--led);
    box-shadow: inset 0 0 0 1px var(--line-subtle);
  }

  .grid .led {
    height: auto;
    aspect-ratio: 1;
  }
</style>
