<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { failureText } from "../../lib/errors";
  import type { AppState } from "../../lib/state/app.svelte";
  import { serverName } from "../connection/wording";
  import { describeOutcome } from "./outcome";
  import { applyingText } from "../effects/present";
  import { identifyBlocked, reference } from "./present";

  let { app, session, device }: { app: AppState; session: SessionSnapshot; device: SessionDevice } = $props();

  let target = $derived(reference(device));
  let blocked = $derived(applyingText(app.view) ?? identifyBlocked(session, device, app.identify?.kind === "running"));
  let result = $derived(app.identify && app.identify.target.device === target ? app.identify : null);
  let zones = $derived(device.descriptor.zones);
  let server = $derived(serverName(session));

  function zoneName(zone: number | null): string {
    return zone === null ? "the whole device" : app.names.zone(device, zone);
  }

  function identify(zone: number | null) {
    if (target !== null && blocked === null) {
      app.identifyTarget({ device: target, zone });
    }
  }
</script>

{#snippet explanation()}
  <p>Identify lights the device or one zone white for 3 seconds. Then PrismForge asks {server} to put back the mode and colours from the last device scan.</p>
  <p>If other software, such as the OpenRGB Effects plugin, changed the lighting after that scan, the older state comes back. Other software can also change the lighting at any time, and PrismForge cannot see that. PrismForge never saves to device memory.</p>
{/snippet}

<section class="identify" aria-labelledby="identify-title">
  <h3 id="identify-title">Identify</h3>
  {#if app.identifiedOnce}
    <details>
      <summary>How identify works</summary>
      <div class="explain">{@render explanation()}</div>
    </details>
  {:else}
    <div class="explain">{@render explanation()}</div>
  {/if}

  <div class="buttons">
    <button
      type="button"
      class="primary"
      disabled={blocked !== null}
      aria-describedby={blocked ? "identify-blocked" : undefined}
      onclick={() => identify(null)}
    >
      Identify device
    </button>
    {#each zones as _, index (index)}
      <button
        type="button"
        disabled={blocked !== null}
        aria-describedby={blocked ? "identify-blocked" : undefined}
        onclick={() => identify(index)}
      >
        Identify zone {app.names.zone(device, index)}
      </button>
    {/each}
  </div>
  {#if blocked}
    <p id="identify-blocked" class="blocked">Identify is off. {blocked}</p>
  {/if}

  <div class="result" role="status" aria-live="polite">
    {#if result?.kind === "running"}
      <p>Lighting {zoneName(result.target.zone)} white for 3 seconds.</p>
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
</section>

<style>
  .identify {
    display: grid;
    gap: 12px;
    padding: 18px 20px;
    border: 1px solid var(--line);
    border-radius: var(--radius-m);
    background: var(--ink-1);
  }

  .explain {
    display: grid;
    gap: 6px;
    color: var(--text-dim);
    max-width: 70ch;
  }

  details summary {
    cursor: pointer;
    color: var(--text-dim);
    width: fit-content;
  }

  details .explain {
    margin-top: 8px;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .blocked {
    color: var(--text-dim);
  }

  .result:empty {
    display: none;
  }

  .outcome {
    display: grid;
    gap: 4px;
    padding: 10px 14px;
    border-left: 3px solid var(--ok);
    background: var(--ink-0);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }

  .outcome.unsettled {
    border-left-color: var(--warning);
  }

  .headline {
    color: var(--text);
    font-weight: 600;
  }
</style>
