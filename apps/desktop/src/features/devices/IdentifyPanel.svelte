<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Details from "../../lib/Details.svelte";
  import { failureText } from "../../lib/errors";
  import type { AppState } from "../../lib/state/app.svelte";
  import { serverName } from "../connection/wording";
  import { describeOutcome } from "./outcome";
  import { reference } from "./present";

  let { app, session, device, blocked }: { app: AppState; session: SessionSnapshot; device: SessionDevice; blocked: string | null } = $props();

  let target = $derived(reference(device));
  let result = $derived(app.identify && app.identify.target.device === target ? app.identify : null);
  let server = $derived(serverName(session));

  function zoneName(zone: number | null): string {
    return zone === null ? "the whole device" : app.names.zone(device, zone);
  }
</script>

{#snippet explanation()}
  <p>Identify lights the device or one zone white for 3 seconds. Then PrismForge asks {server} to put back the mode and colours from the last device scan.</p>
  <p>If other software, such as the OpenRGB Effects plugin, changed the lighting after that scan, the older state comes back. Other software can also change the lighting at any time, and PrismForge cannot see that. PrismForge never saves to device memory.</p>
{/snippet}

<div class="identify">
  {#if blocked}
    <p id="identify-blocked" class="blocked">Identify is off. {blocked}</p>
  {/if}

  <div class="result" role="status" aria-live="polite">
    {#if result?.kind === "running"}
      <p class="outcome">Lighting {zoneName(result.target.zone)} white for 3 seconds.</p>
    {:else if result?.kind === "finished"}
      {@const text = describeOutcome(result.outcome, server)}
      <div class="outcome" class:unsettled={!text.settled}>
        <p class="headline">{text.headline}</p>
        {#each text.details as detail (detail)}
          <p>{detail}</p>
        {/each}
      </div>
    {:else if result?.kind === "failed"}
      <div class="outcome unsettled">
        <p class="headline">Identify did not run.</p>
        <p>{failureText(result.error)}</p>
      </div>
    {/if}
  </div>

  <p class="explain">Identify lights it white for 3 seconds, then asks {server} to put back the earlier lighting.</p>
  <Details summary="How identify works">{@render explanation()}</Details>
</div>

<style>
  .identify {
    display: grid;
    gap: var(--space-2);
  }

  .explain {
    display: grid;
    gap: var(--space-1);
    color: var(--text-dim);
    font-size: var(--font-s);
    max-width: 72ch;
  }

  .blocked {
    color: var(--text-dim);
  }

  .result:empty {
    display: none;
  }

  .outcome {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--ok);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }

  .outcome.unsettled {
    border-left-color: var(--warning);
  }

  .headline {
    color: var(--text);
    font-weight: var(--weight-strong);
  }
</style>
