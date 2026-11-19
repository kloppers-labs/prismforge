<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { reference } from "../devices/present";
  import FailureNotice from "./FailureNotice.svelte";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    session,
    device,
    held,
  }: { effects: EffectsState; session: SessionSnapshot; device: SessionDevice; held: string | null } = $props();

  let ref = $derived(reference(device));
  let identifying = $derived(ref !== null && session.identify?.device === ref);
  let program = $state("");

  function declare(event: SubmitEvent) {
    event.preventDefault();
    if (ref !== null) {
      effects.declareExternal(ref, program);
    }
  }
</script>

<section class="external" aria-labelledby="external-title">
  <h4 id="external-title">Used by another program</h4>
  <p>
    PrismForge cannot see other programs that write to a device, such as the OpenRGB Effects plugin or Windows Dynamic Lighting,
    and it cannot stop them. If two programs write to one device, the lights flicker or the last writer wins.
  </p>
  <p>
    If another program controls this device, say so here. PrismForge then stops its own effect on this device, without
    putting back the earlier lighting, and sends nothing to it until you hand it back.
  </p>
  <form class="declare" onsubmit={declare}>
    <label for="program-name">Program name (optional)</label>
    <input id="program-name" type="text" maxlength="60" autocomplete="off" bind:value={program} placeholder="another program" />
    <button
      type="submit"
      disabled={ref === null || identifying || held !== null}
      aria-describedby={held ? "external-held" : identifying ? "external-blocked" : undefined}
    >
      Leave this device to another program
    </button>
  </form>
  {#if identifying}
    <p id="external-blocked" class="hint">Off while an identify runs on this device.</p>
  {/if}
  {#if held}
    <p id="external-held" class="hint">Off now. {held}</p>
  {/if}
  {#if effects.externalProblem}
    <FailureNotice error={effects.externalProblem} />
  {/if}
</section>

<style>
  .external {
    display: grid;
    gap: var(--space-2);
    color: var(--text-dim);
  }

  h4 {
    margin: 0;
    color: var(--text);
    font-size: var(--font-m);
  }

  .external p {
    max-width: 72ch;
  }

  .hint {
    font-size: var(--font-s);
  }

  .declare {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
  }

  label {
    color: var(--text);
  }

  input {
    width: 24ch;
    max-width: 100%;
  }
</style>
