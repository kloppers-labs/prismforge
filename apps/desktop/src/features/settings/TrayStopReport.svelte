<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { StopReport } from "../../lib/api/bindings/StopReport";
  import { serverName } from "../connection/wording";
  import { stopResultText } from "./wording";

  let {
    report,
    session,
    label,
  }: { report: StopReport; session: SessionSnapshot | null; label: (device: string, reported: string) => string } = $props();

  let server = $derived(session ? serverName(session) : "the server");
  let failed = $derived(report.stops.some((stop) => "Failed" in stop.result));
</script>

{#if report.stops.length > 0}
  <ul>
    {#each report.stops as stop (stop.device)}
      <li>
        <span class="device">{label(stop.device, stop.label)}</span>
        {#each stopResultText(stop.result, server) as line (line)}
          <span>{line}</span>
        {/each}
      </li>
    {/each}
  </ul>
{/if}
{#if failed}
  <p class="hint">PrismForge does not try again by itself. Stop the remaining effects on the Effects screen.</p>
{/if}

<style>
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: var(--space-1);
  }

  li {
    display: grid;
    gap: 2px;
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .device {
    color: var(--text);
    font-weight: var(--weight-strong);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
