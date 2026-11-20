<script lang="ts">
  import { tick } from "svelte";

  let {
    id,
    label,
    value,
    min,
    max,
    step = 1,
    unit = "",
    describedby,
    onchange,
  }: {
    id: string;
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    unit?: string;
    describedby?: string;
    onchange: (value: number) => void;
  } = $props();
</script>

<div class="field">
  <label for={id}>{label}</label>
  <span class="input">
    <input
      {id}
      type="number"
      inputmode="numeric"
      {min}
      {max}
      {step}
      {value}
      aria-describedby={describedby}
      onchange={async (event) => {
        const input = event.currentTarget;
        const next = Number(input.value);
        if (input.value.trim() !== "" && Number.isFinite(next)) {
          onchange(next);
        }
        await tick();
        input.value = String(value);
      }}
    />
    {#if unit}<span class="unit" aria-hidden="true">{unit}</span>{/if}
  </span>
</div>

<style>
  .field {
    display: grid;
    gap: var(--space-1);
  }

  label {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .input {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  input {
    width: 100%;
    min-width: 0;
  }

  .unit {
    color: var(--text-dim);
  }
</style>
