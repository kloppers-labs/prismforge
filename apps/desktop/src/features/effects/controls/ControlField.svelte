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
    onchange,
  }: { control: ControlDefinition; field: Field; problem: string | null; onchange: (field: Field) => void } = $props();

  let id = $derived(`control-${control.id}`);
  let kind = $derived(control.kind);
</script>

{#if field.kind === "color"}
  <div class="described">
    <ColorField {id} label={control.label} text={field.text} {problem} describedBy="{id}-hint" onchange={(text) => onchange({ kind: "color", text })} />
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
    onchange={(texts) => onchange({ kind: "palette", texts })}
  />
{:else if field.kind === "direction"}
  <DirectionField {id} label={control.label} description={control.description} value={field.value} {problem} onchange={(value) => onchange({ kind: "direction", value })} />
{:else if field.kind === "number"}
  <RangeField
    {id}
    label={control.label}
    description={control.description}
    text={field.text}
    range={numberKind(control)}
    {problem}
    onchange={(text) => onchange({ kind: "number", text })}
  />
{/if}

<style>
  .described {
    display: grid;
    gap: 6px;
  }

  .hint {
    color: var(--text-faint);
    font-size: var(--font-s);
  }
</style>
