<script lang="ts">
  import type { ControlDefinition } from "../../../lib/api/bindings/ControlDefinition";
  import { numberKind, type Field } from "../values";
  import ColorField from "./ColorField.svelte";
  import DirectionField from "./DirectionField.svelte";
  import PaletteField from "./PaletteField.svelte";
  import RangeField from "./RangeField.svelte";

  let {
    control,
    field,
    problem,
    mixed = false,
    onchange,
  }: { control: ControlDefinition; field: Field; problem: string | null; mixed?: boolean; onchange: (field: Field) => void } = $props();

  let id = $derived(`control-${control.id}`);
  let also = $derived(mixed ? `mixed-${control.id}` : null);
  let kind = $derived(control.kind);
</script>

{#if mixed}
  <p id="mixed-{control.id}" class="mixed">Mixed. The targets have different values for {control.label.toLowerCase()}; the value shown is the first target's. Apply sends it to all of them.</p>
{/if}
{#if field.kind === "color"}
  <div class="described">
    <ColorField {id} label={control.label} text={field.text} {problem} describedBy={also ? `${also} ${id}-hint` : `${id}-hint`} onchange={(text) => onchange({ kind: "color", text })} />
    <p id="{id}-hint" class="hint">{control.description}</p>
  </div>
{:else if field.kind === "palette" && "Palette" in kind}
  <PaletteField
    {id}
    label={control.label}
    description={control.description}
    texts={field.texts}
    min={kind.Palette.min_colors}
    max={kind.Palette.max_colors}
    {problem}
    {also}
    onchange={(texts) => onchange({ kind: "palette", texts })}
  />
{:else if field.kind === "direction"}
  <DirectionField {id} label={control.label} description={control.description} value={field.value} {problem} {also} onchange={(value) => onchange({ kind: "direction", value })} />
{:else if field.kind === "number"}
  <RangeField
    {id}
    label={control.label}
    description={control.description}
    text={field.text}
    range={numberKind(control)}
    {problem}
    {also}
    onchange={(text) => onchange({ kind: "number", text })}
  />
{/if}

<style>
  .described {
    display: grid;
    gap: var(--space-2);
  }

  .mixed {
    color: var(--warning);
    font-size: var(--font-s);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
