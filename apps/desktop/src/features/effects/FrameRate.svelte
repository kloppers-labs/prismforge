<script lang="ts">
  import type { Bounds } from "../../lib/api/bindings/Bounds";
  import { failureText } from "../../lib/errors";
  import type { EffectsState } from "./state.svelte";

  let {
    effects,
    range,
    current,
    held,
  }: { effects: EffectsState; range: Bounds; current: number; held: string | null } = $props();

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

<div class="rate">
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
    <button type="submit" disabled={held !== null} aria-describedby={held ? "frame-rate-held" : undefined}>Set frame rate</button>
  </form>
  {#if held}
    <p id="frame-rate-held" class="hint">Off now. {held}</p>
  {/if}
  {#if problem}
    <p id="frame-rate-problem" class="problem" role="alert">{problem}</p>
  {/if}
  <div class="result" role="status" aria-label="Frame rate result">
    {#if feedback?.kind === "set"}
      <p>
        {feedback.used === feedback.requested
          ? `PrismForge set the frame rate to ${feedback.used} frames per second.`
          : `You asked for ${feedback.requested} frames per second. PrismForge set the nearest allowed rate, ${feedback.used}.`}
      </p>
    {:else if feedback?.kind === "failed"}
      <p>{failureText(feedback.error)}</p>
    {/if}
  </div>
</div>

<style>
  .rate {
    display: grid;
    gap: var(--space-2);
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .rate p {
    max-width: 72ch;
  }

  .hint {
    font-size: var(--font-s);
  }

  form {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
  }

  label {
    color: var(--text);
  }

  input {
    width: 8ch;
  }

  .problem {
    color: var(--problem);
    font-size: var(--font-s);
  }

  .result:empty {
    display: none;
  }
</style>
