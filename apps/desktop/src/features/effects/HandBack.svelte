<script lang="ts">
  import Details from "../../lib/Details.svelte";
  import Notice from "../../lib/Notice.svelte";
  import FailureNotice from "./FailureNotice.svelte";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    device,
    label,
    held,
  }: { effects: EffectsState; device: string; label: string; held: string | null } = $props();
</script>

<Notice tone="warning" label="Left to another program" title="Left to {label}">
  <p>You said {label} controls this device. PrismForge sends nothing to it.</p>
  <p>Before you hand it back, stop this device's effect in {label}. PrismForge cannot check this.</p>
  <Details>
    <p>Handing back only records your statement that {label} no longer writes to this device. If two programs write to one device, the lights flicker or the last writer wins.</p>
  </Details>
  {#if held}<p id="handback-held">Off now. {held}</p>{/if}
  {#if effects.externalProblem}<FailureNotice error={effects.externalProblem} />{/if}
  {#snippet actions()}
    <button
      type="button"
      class="small"
      data-mirror="target.hand-back"
      disabled={held !== null}
      aria-describedby={held ? "handback-held" : undefined}
      onclick={() => effects.clearExternal(device)}
    >
      Hand back to PrismForge
    </button>
  {/snippet}
</Notice>
