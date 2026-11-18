<script lang="ts">
  import type { Bounds } from "../../lib/api/bindings/Bounds";
  import { failureText } from "../../lib/errors";
  import type { EffectsState } from "./state.svelte";

  let { effects, range, current }: { effects: EffectsState; range: Bounds; current: number } = $props();

  let text = $state("");
  let problem = $state<string | null>(null);
  let feedback = $derived(effects.frameRateFeedback);

  $effect.pre(() => {
    text = String(current);
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = text.trim();
    if (!/^\d+$/.test(trimmed)) {
      problem = "Enter a whole number.";
      return;
    }
    problem = null;
    effects.setFrameRate(Number(trimmed));
  }
</script>

<section class="rate" aria-labelledby="rate-title">
  <h3 id="rate-title">Frame rate</h3>
  <p>
    From {range.min} to {range.max} frames per second. Now {current}. The rate applies to every PrismForge effect; an
    effect that does not change is written once.
  </p>
  <p>Higher rates use more processor time. OpenRGB and the device may not keep up.</p>
  <form novalidate onsubmit={submit}>
    <label for="frame-rate">Frames per second</label>
    <input
      id="frame-rate"
      class="numeric"
      type="number"
      inputmode="numeric"
      min={range.min}
      max={range.max}
      value={text}
      oninput={(event) => (text = event.currentTarget.value)}
      aria-invalid={problem ? "true" : undefined}
      aria-describedby={problem ? "frame-rate-problem" : undefined}
    />
    <button type="submit">Set frame rate</button>
  </form>
  {#if problem}
    <p id="frame-rate-problem" class="problem">{problem}</p>
  {/if}
  <div class="result" role="status" aria-label="Frame rate result">
    {#if feedback?.kind === "done"}
      <p>{feedback.text}</p>
    {:else if feedback?.kind === "failed"}
      <p>{failureText(feedback.error)}</p>
    {/if}
  </div>
</section>

<style>
  .rate {
    display: grid;
    gap: 8px;
    color: var(--text-dim);
  }

  .rate p {
    max-width: 72ch;
  }

  form {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  label {
    color: var(--text);
  }

  input {
    width: 8ch;
  }

  .problem {
    color: var(--problem);
    font-size: 13px;
  }

  .result:empty {
    display: none;
  }
</style>
