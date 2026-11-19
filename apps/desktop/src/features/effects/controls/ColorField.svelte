<script lang="ts">
  import { COLOR_EXAMPLE, parseHex, toHex } from "../values";

  let {
    id,
    label,
    text,
    problem = null,
    describedBy = null,
    onchange,
  }: {
    id: string;
    label: string;
    text: string;
    problem?: string | null;
    describedBy?: string | null;
    onchange: (text: string) => void;
  } = $props();

  let color = $derived(parseHex(text));
  let hex = $derived(color ? toHex(color) : null);
  let described = $derived([problem ? `${id}-problem` : null, describedBy].filter(Boolean).join(" ") || undefined);
</script>

<div class="color">
  <label for={id}>{label}</label>
  <div class="row">
    <span class="swatch" class:unset={!hex} style:--swatch={hex ?? "transparent"}>
      <input
        type="color"
        aria-label="{label} picker"
        value={hex ?? "#000000"}
        aria-describedby={described}
        oninput={(event) => onchange(event.currentTarget.value)}
      />
    </span>
    <input
      {id}
      class="hex"
      type="text"
      value={text}
      placeholder={COLOR_EXAMPLE}
      spellcheck="false"
      autocomplete="off"
      aria-invalid={problem ? "true" : undefined}
      aria-describedby={described}
      oninput={(event) => onchange(event.currentTarget.value)}
    />
  </div>
  {#if problem}
    <p id="{id}-problem" class="problem">{problem}</p>
  {/if}
</div>

<style>
  .color {
    display: grid;
    gap: var(--space-2);
  }

  label {
    font-weight: var(--weight-strong);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .swatch {
    position: relative;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-s);
    background: var(--swatch);
    box-shadow: inset 0 0 0 1px var(--line-control);
    forced-color-adjust: none;
  }

  .swatch.unset {
    background: repeating-linear-gradient(45deg, var(--surface-2) 0 5px, var(--surface-3) 5px 10px);
  }

  .swatch input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    padding: 0;
    border: 0;
  }

  .swatch:has(input:focus-visible) {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }

  .hex {
    width: 9ch;
    font-variant-numeric: tabular-nums;
  }

  .problem {
    color: var(--problem);
    font-size: var(--font-s);
  }
</style>
