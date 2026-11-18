<script lang="ts">
  import type { NumberKind } from "../values";

  let {
    id,
    label,
    description,
    text,
    range,
    problem = null,
    onchange,
  }: {
    id: string;
    label: string;
    description: string;
    text: string;
    range: NumberKind;
    problem?: string | null;
    onchange: (text: string) => void;
  } = $props();

  let slider = $derived(/^\d+$/.test(text.trim()) ? Number(text) : range.default);
  let described = $derived(`${problem ? `${id}-problem ` : ""}${id}-hint`);
  let span = $derived(`${range.min} to ${range.max}${range.unit === "%" ? "%" : range.unit ? ` ${range.unit}` : ""}`);
</script>

<div class="range">
  <label for={id}>{label}</label>
  <div class="row">
    <input
      type="range"
      aria-label={label}
      min={range.min}
      max={range.max}
      step={range.step}
      value={slider}
      aria-describedby={described}
      oninput={(event) => onchange(event.currentTarget.value)}
    />
    <input
      {id}
      class="exact numeric"
      type="number"
      inputmode="numeric"
      min={range.min}
      max={range.max}
      step={range.step}
      value={text}
      aria-invalid={problem ? "true" : undefined}
      aria-describedby={described}
      oninput={(event) => onchange(event.currentTarget.value)}
    />
    {#if range.unit}<span class="unit">{range.unit}</span>{/if}
  </div>
  <p id="{id}-hint" class="hint">{description} From {span}.</p>
  {#if problem}
    <p id="{id}-problem" class="problem">{problem}</p>
  {/if}
</div>

<style>
  .range {
    display: grid;
    gap: 6px;
  }

  label {
    font-weight: 550;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  input[type="range"] {
    flex: 1;
    max-width: 320px;
    accent-color: var(--text);
  }

  .exact {
    width: 8ch;
  }

  .unit {
    color: var(--text-dim);
  }

  .hint {
    color: var(--text-faint);
    font-size: 13px;
  }

  .problem {
    color: var(--problem);
    font-size: 13px;
  }
</style>
