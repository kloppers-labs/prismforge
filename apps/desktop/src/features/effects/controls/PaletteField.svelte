<script lang="ts">
  import ColorField from "./ColorField.svelte";

  let {
    id,
    label,
    description,
    texts,
    min,
    max,
    problem = null,
    onchange,
  }: {
    id: string;
    label: string;
    description: string;
    texts: string[];
    min: number;
    max: number;
    problem?: string | null;
    onchange: (texts: string[]) => void;
  } = $props();

  function replace(index: number, text: string) {
    onchange(texts.map((current, at) => (at === index ? text : current)));
  }

  function move(index: number, by: number) {
    const next = [...texts];
    const [moved] = next.splice(index, 1);
    next.splice(index + by, 0, moved ?? "");
    onchange(next);
  }

  function remove(index: number) {
    onchange(texts.filter((_, at) => at !== index));
  }

  function add() {
    onchange([...texts, texts.at(-1) ?? ""]);
  }
</script>

<fieldset class="palette" aria-describedby="{id}-hint{problem ? ` ${id}-problem` : ''}">
  <legend>{label}</legend>
  <p id="{id}-hint" class="hint">{description} Use {min} to {max} colours, in order.</p>
  <ol>
    {#each texts as text, index (index)}
      <li>
        <ColorField id="{id}-{index}" label="Colour {index + 1}" {text} onchange={(next) => replace(index, next)} />
        <div class="order">
          <button type="button" disabled={index === 0} onclick={() => move(index, -1)} aria-label="Move colour {index + 1} up">Up</button>
          <button type="button" disabled={index === texts.length - 1} onclick={() => move(index, 1)} aria-label="Move colour {index + 1} down">Down</button>
          <button type="button" disabled={texts.length <= min} onclick={() => remove(index)} aria-label="Remove colour {index + 1}">Remove</button>
        </div>
      </li>
    {/each}
  </ol>
  <button type="button" class="add" disabled={texts.length >= max} onclick={add}>Add colour</button>
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
    gap: 8px;
  }

  legend {
    font-weight: var(--weight-strong);
    padding: 0;
    margin-bottom: 4px;
  }

  .hint {
    color: var(--text-faint);
    font-size: var(--font-s);
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 10px;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 10px 16px;
  }

  .order {
    display: flex;
    gap: 4px;
  }

  .order button {
    padding: 5px 10px;
    font-size: var(--font-s);
  }

  .add {
    justify-self: start;
  }

  .problem {
    color: var(--problem);
    font-size: var(--font-s);
  }
</style>
