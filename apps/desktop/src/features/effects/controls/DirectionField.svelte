<script lang="ts">
  import type { Direction } from "../../../lib/api/bindings/Direction";

  let {
    id,
    label,
    description,
    value,
    problem = null,
    onchange,
  }: {
    id: string;
    label: string;
    description: string;
    value: Direction;
    problem?: string | null;
    onchange: (value: Direction) => void;
  } = $props();

  const options: { value: Direction; label: string; detail: string }[] = [
    { value: "Forward", label: "Forward", detail: "from the first LED towards the last" },
    { value: "Backward", label: "Backward", detail: "from the last LED towards the first" },
  ];
</script>

<fieldset aria-describedby="{problem ? `${id}-problem ` : ''}{id}-hint">
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
    gap: 6px;
  }

  legend {
    font-weight: 550;
    padding: 0;
    margin-bottom: 4px;
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 20px;
  }

  label {
    display: flex;
    gap: 6px;
    align-items: baseline;
  }

  .problem {
    color: var(--problem);
    font-size: 13px;
  }

  .detail,
  .hint {
    color: var(--text-faint);
    font-size: 13px;
  }
</style>
