<script lang="ts">
  import type { Direction } from "../../../lib/api/bindings/Direction";

  let {
    id,
    label,
    description,
    value,
    problem = null,
    also = null,
    onchange,
  }: {
    id: string;
    label: string;
    description: string;
    value: Direction;
    problem?: string | null;
    also?: string | null;
    onchange: (value: Direction) => void;
  } = $props();

  const options: { value: Direction; label: string; detail: string }[] = [
    { value: "Forward", label: "Forward", detail: "from the first LED towards the last" },
    { value: "Backward", label: "Backward", detail: "from the last LED towards the first" },
  ];
</script>

<fieldset aria-describedby="{also ? `${also} ` : ''}{problem ? `${id}-problem ` : ''}{id}-hint">
  <legend>{label}</legend>
  <div class="options">
    {#each options as option (option.value)}
      <label>
        <input type="radio" name={id} value={option.value} checked={value === option.value} onchange={() => onchange(option.value)} />
        <span>{option.label}</span>
        <span class="detail">{option.detail}</span>
      </label>
    {/each}
  </div>
  <p id="{id}-hint" class="hint">{description}</p>
  {#if problem}
    <p id="{id}-problem" class="problem">{problem}</p>
  {/if}
</fieldset>

<style>
  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
    display: grid;
    gap: var(--space-2);
  }

  legend {
    font-weight: var(--weight-strong);
    padding: 0;
    margin-bottom: var(--space-1);
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2) var(--space-4);
  }

  label {
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
  }

  .problem {
    color: var(--problem);
    font-size: var(--font-s);
  }

  .detail,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
