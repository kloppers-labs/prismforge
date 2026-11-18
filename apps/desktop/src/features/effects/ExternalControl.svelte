<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { reference } from "../devices/present";
  import FailureNotice from "./FailureNotice.svelte";
  import { declaredBy } from "./present";
  import type { EffectsState } from "./state.svelte";

  let { effects, session, device }: { effects: EffectsState; session: SessionSnapshot; device: SessionDevice } = $props();

  let ref = $derived(reference(device));
  let label = $derived(ref === null ? null : declaredBy(session, ref));
  let identifying = $derived(ref !== null && session.identify?.device === ref);
  let program = $state("");
  let feedback = $derived(effects.externalFeedback);

  function declare(event: SubmitEvent) {
    event.preventDefault();
    if (ref !== null) {
      effects.declareExternal(ref, program);
    }
  }
</script>

<section class="external" aria-labelledby="external-title">
  <h3 id="external-title">Another program</h3>
  {#if label !== null}
    <p class="headline">You said {label} controls this device. PrismForge sends nothing to it.</p>
    <p>
      Before you hand it back, stop this device's effect in {label}. PrismForge cannot check this. If two programs write to
      one device, the lights flicker or the last writer wins.
    </p>
    <p class="hint">Handing back only records your statement that {label} no longer writes to this device.</p>
    <div>
      <button type="button" disabled={ref === null} onclick={() => ref !== null && effects.clearExternal(ref)}>
        Hand back to PrismForge
      </button>
    </div>
  {:else}
    <p>
      PrismForge cannot see other programs that write to a device, such as the OpenRGB Effects plugin, and it cannot stop
      them. If two programs write to one device, the lights flicker or the last writer wins.
    </p>
    <p>
      If another program controls this device, say so here. PrismForge then stops its own effect on this device, without
      putting back the earlier lighting, and sends nothing to it until you hand it back.
    </p>
    <form class="declare" onsubmit={declare}>
      <label for="program-name">Program name (optional)</label>
      <input id="program-name" type="text" maxlength="60" autocomplete="off" bind:value={program} placeholder="Another program" />
      <button type="submit" disabled={ref === null || identifying} aria-describedby={identifying ? "external-blocked" : undefined}>
        Another program controls this device
      </button>
    </form>
    {#if identifying}
      <p id="external-blocked" class="hint">Off while an identify runs on this device.</p>
    {/if}
  {/if}
  {#if feedback?.kind === "failed"}
    <FailureNotice error={feedback.error} />
  {/if}
</section>

<style>
  .external {
    display: grid;
    gap: 10px;
    color: var(--text-dim);
  }

  .external p {
    max-width: 72ch;
  }

  .headline {
    color: var(--text);
    font-weight: 600;
  }

  .hint {
    color: var(--text-faint);
    font-size: 13px;
  }

  .declare {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
  }

  label {
    color: var(--text);
  }

  input {
    width: 24ch;
  }
</style>
