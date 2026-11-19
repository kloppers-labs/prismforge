<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { capitalise, serverName } from "../connection/wording";
  import { reference } from "../devices/present";
  import FailureNotice from "./FailureNotice.svelte";
  import { modeBlocked, outputOf } from "./present";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    session,
    device,
    held,
  }: { effects: EffectsState; session: SessionSnapshot; device: SessionDevice; held: string | null } = $props();

  let ref = $derived(reference(device));
  let capabilities = $derived(device.descriptor.capabilities);
  let modes = $derived(
    capabilities.modes.map((mode, index) => ({ index, name: mode.name })).filter((mode) => mode.index !== capabilities.direct_mode),
  );
  let blocked = $derived(held ?? modeBlocked(session, device));
  let owner = $derived(ref === null ? null : (outputOf(session, ref)?.owner ?? null));
  let selected = $derived(owner !== null && typeof owner === "object" && "HardwareMode" in owner ? owner.HardwareMode.mode : null);
  let feedback = $derived(effects.modeFeedback);
</script>

<section class="modes" aria-labelledby="modes-title">
  <h3 id="modes-title">Hardware modes</h3>
  <p class="explain">
    A hardware mode runs on the device itself, also when PrismForge is closed. Selecting one stops the PrismForge effect
    on this device and does not put back the earlier lighting. PrismForge asks {serverName(session)} to use the mode with
    the settings the device reported. It changes no settings and never saves to device memory.
  </p>
  {#if modes.length === 0}
    <p class="hint">This device reports no hardware modes.</p>
  {:else}
    <ul>
      {#each modes as mode (mode.index)}
        <li>
          <button
            type="button"
            disabled={blocked !== null || ref === null}
            aria-describedby={blocked ? "modes-blocked" : undefined}
            onclick={() => ref !== null && effects.selectMode(ref, mode.index, mode.name)}
          >
            Use {mode.name}
          </button>
          {#if mode.index === selected}
            <span class="tag">Selected in PrismForge</span>
          {:else if mode.index === capabilities.active_mode}
            <span class="tag">Active in the last device scan</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if blocked}
    <p id="modes-blocked" class="hint">Hardware modes are off. {blocked}</p>
  {/if}
  {#if feedback?.kind === "failed"}
    <FailureNotice error={feedback.error} />
  {/if}
  <div class="result" role="status" aria-label="Hardware mode result">
    {#if feedback?.kind === "selected"}
      <p>{capitalise(feedback.server)} accepted the request to use hardware mode {feedback.name}. PrismForge cannot confirm what the device shows.</p>
    {/if}
  </div>
</section>

<style>
  .modes {
    display: grid;
    gap: 10px;
  }

  .explain,
  .hint {
    color: var(--text-dim);
    max-width: 72ch;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tag {
    font-size: 12px;
    color: var(--text-dim);
  }

  .result {
    color: var(--text-dim);
  }

  .result:empty {
    display: none;
  }
</style>
