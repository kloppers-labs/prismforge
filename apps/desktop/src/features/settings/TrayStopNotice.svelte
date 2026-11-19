<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { StopReport } from "../../lib/api/bindings/StopReport";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import { serverName } from "../connection/wording";
  import { stopResultText } from "./wording";

  let {
    report,
    session,
    label,
    ondismiss,
  }: { report: StopReport; session: SessionSnapshot | null; label: (device: string, reported: string) => string; ondismiss: () => void } = $props();

  let server = $derived(session ? serverName(session) : "the server");
  let failed = $derived(report.stops.some((stop) => "Failed" in stop.result));
</script>

<section class="notice {failed ? 'warning' : 'done'}" role="status" aria-labelledby="tray-stop-title">
  <ToneIcon tone={failed ? "warning" : "ok"} />
  <div class="body">
    <h2 id="tray-stop-title">
      {#if report.stops.length === 0}
        Stop all PrismForge effects, from the tray: no PrismForge effect was running.
      {:else if failed}
        Stop all PrismForge effects, from the tray: some effects were not stopped.
      {:else}
        Stop all PrismForge effects, from the tray: every effect was stopped.
      {/if}
    </h2>
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
    {#if failed}
      <p class="hint">PrismForge does not try again by itself. Stop the remaining effects on the Effects screen.</p>
    {/if}
  </div>
  <button type="button" class="small" onclick={ondismiss}>Dismiss</button>
</section>

<style>
  .notice {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 10px;
    align-items: start;
    padding: 10px 20px;
    border-bottom: 1px solid var(--line);
    background: var(--ink-1);
  }

  .notice :global(svg) {
    margin-top: 3px;
  }

  .notice.warning {
    background: #2a2519;
  }

  h2 {
    font-size: 14px;
    font-weight: 600;
  }

  .body {
    display: grid;
    gap: 6px;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 4px;
  }

  li {
    display: grid;
    gap: 2px;
    color: var(--text-dim);
    font-size: 13px;
  }

  .device {
    color: var(--text);
    font-weight: 600;
  }

  .hint {
    color: var(--text-dim);
    font-size: 13px;
  }

  .small {
    padding: 2px 10px;
    font-size: 13px;
  }
</style>
