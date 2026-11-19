<script lang="ts">
  import type { ComponentId } from "../../lib/api/bindings/ComponentId";
  import type { Rgb } from "../../lib/api/bindings/Rgb";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { contextMenu, type Menu } from "../../lib/shell/menu.svelte";
  import { outlineOf, placementOf } from "./edits";
  import { editAction, keyHelp, type EditAction } from "./keys";
  import { firstLedText, kindText, outlinePlaceText, outputLine, placeText, wiringTexts } from "./present";
  import type { SetupState } from "./state.svelte";
  import { outputOf } from "./wiring";

  let {
    setup,
    session,
    names,
    colors,
    onaction,
    menu,
  }: {
    setup: SetupState;
    session: SessionSnapshot;
    names: Names;
    colors: Record<ComponentId, Rgb[]> | null;
    onaction: (action: EditAction) => void;
    menu: (id: ComponentId) => Menu | null;
  } = $props();

  const OUTLINE_WORDS = { Case: "a case", Desk: "a desk", Monitor: "a monitor", Pc: "a PC" } as const;

  let list: HTMLElement | undefined = $state();
  let focused = $state<ComponentId | null>(null);
  let rows = $derived([...setup.placed, ...setup.drawn]);
  let viewName = $derived(setup.view?.name ?? "");
  let count = $derived(setup.selection.length);
  let stop = $derived((focused !== null && rows.includes(focused) ? focused : null) ?? rows.find((id) => setup.selection.includes(id)) ?? rows[0] ?? null);

  function focusRow(id: ComponentId) {
    focused = id;
    list?.querySelector<HTMLElement>(`[data-row="${CSS.escape(id)}"]`)?.focus();
  }

  function onkey(event: KeyboardEvent, id: ComponentId) {
    const at = rows.indexOf(id);
    const command = event.ctrlKey || event.metaKey;
    const moves: Record<string, number> = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: rows.length - 1 };
    if (event.key in moves && !event.altKey && !command) {
      const next = rows[Math.min(Math.max(moves[event.key] ?? at, 0), rows.length - 1)];
      if (!next) {
        return;
      }
      event.preventDefault();
      if (event.shiftKey && !setup.selection.includes(next)) {
        setup.select([...setup.selection, next]);
      }
      focusRow(next);
      return;
    }
    if (event.key === " " && !command) {
      event.preventDefault();
      setup.toggle(id);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      setup.select([id]);
      return;
    }
    if (command && event.key.toLowerCase() === "a") {
      event.preventDefault();
      setup.select(rows);
      return;
    }
    if (event.key === "Escape" && count > 0) {
      event.preventDefault();
      setup.select([]);
      return;
    }
    const action = editAction(event);
    if (action) {
      event.preventDefault();
      event.stopPropagation();
      onaction(action);
    }
  }

  function onclick(event: MouseEvent, id: ComponentId) {
    focused = id;
    if (event.target instanceof Element && event.target.closest(".box")) {
      setup.toggle(id);
    } else if (event.ctrlKey || event.metaKey) {
      setup.toggle(id);
    } else if (event.shiftKey) {
      const from = rows.indexOf(setup.selection.at(-1) ?? id);
      const to = rows.indexOf(id);
      const [low, high] = from <= to ? [from, to] : [to, from];
      setup.select([...setup.selection, ...rows.slice(low, high + 1)]);
    } else {
      setup.select([id]);
    }
  }

  function swatch(color: Rgb): string {
    return `rgb(${color.r} ${color.g} ${color.b})`;
  }
</script>

<section class="twin" aria-labelledby="twin-title">
  <div class="head">
    <h2 id="twin-title" class="title">In the {viewName} view</h2>
    <div class="tools">
      <button type="button" class="small quiet" disabled={rows.length === 0} onclick={() => setup.select(rows)}>Select all</button>
      <button type="button" class="small quiet" disabled={count === 0} onclick={() => setup.select([])}>Clear</button>
    </div>
  </div>
  <p id="twin-count" class="count" role="status">{count === 0 ? "No component selected" : count === 1 ? "1 component selected" : `${count} components selected`}</p>
  {#if rows.length === 0}
    <p class="empty">Nothing is placed in the {viewName} view yet. Place a detected device below, or start from a template.</p>
  {/if}
  <div
    class="list"
    role="listbox"
    aria-multiselectable="true"
    aria-label="Components in the {viewName} view"
    aria-describedby="twin-count twin-keys"
    bind:this={list}
  >
    {#each rows as id (id)}
      {@const outline = outlineOf(setup.draft, setup.tab, id)}
      {@const component = setup.draft.components.find((each) => each.id === id)}
      {@const placement = placementOf(setup.draft, setup.tab, id)}
      {@const output = outputOf(setup.draft, id)}
      {@const selected = setup.selection.includes(id)}
      {@const shown = colors?.[id] ?? null}
      {#if component && placement}
        <div
          class="pick option"
          role="option"
          tabindex={stop === id ? 0 : -1}
          aria-selected={selected}
          aria-labelledby="row-name-{id}"
          aria-describedby="row-about-{id}"
          aria-keyshortcuts="Shift+F10"
          data-row={id}
          data-mirror="component.select"
          onclick={(event) => onclick(event, id)}
          onkeydown={(event) => onkey(event, id)}
          onfocus={() => (focused = id)}
          use:contextMenu={() => menu(id)}
        >
          <span class="box" class:checked={selected} aria-hidden="true">{selected ? "✓" : ""}</span>
          <span class="body">
            <span class="line">
              <span id="row-name-{id}" class="pick-name">{component.name}</span>
              <span class="pick-meta">{kindText(component.geometry, component.frame !== undefined)}</span>
            </span>
            <span id="row-about-{id}" class="about">
              <span class:unwired={!output}>{output ? outputLine(output, names, session) : "Not wired to an output"}</span>
              {#each wiringTexts(setup.draft, id, names, session) as text (text)}
                <span>{text}</span>
              {/each}
              <span>{firstLedText(component.geometry, placement)}</span>
              <span>{placeText(placement)}</span>
            </span>
            {#if shown}
              <span class="swatches" aria-hidden="true">
                {#each shown.slice(0, 24) as color, index (index)}
                  <span class="swatch" style:background={swatch(color)}></span>
                {/each}
              </span>
            {/if}
          </span>
        </div>
      {:else if outline}
        {@const selected = setup.selection.includes(id)}
        <div
          class="pick option"
          role="option"
          tabindex={stop === id ? 0 : -1}
          aria-selected={selected}
          aria-labelledby="row-name-{id}"
          aria-describedby="row-about-{id}"
          aria-keyshortcuts="Shift+F10"
          data-row={id}
          data-mirror="outline.select"
          onclick={(event) => onclick(event, id)}
          onkeydown={(event) => onkey(event, id)}
          onfocus={() => (focused = id)}
          use:contextMenu={() => menu(id)}
        >
          <span class="box" class:checked={selected} aria-hidden="true">{selected ? "✓" : ""}</span>
          <span class="body">
            <span class="line">
              <span id="row-name-{id}" class="pick-name">{outline.name}</span>
              <span class="pick-meta">Outline</span>
            </span>
            <span id="row-about-{id}" class="about">
              <span>Outline of {OUTLINE_WORDS[outline.kind]}. No LEDs: it is drawn only and nothing is wired to it.</span>
              <span>{outlinePlaceText(outline)}</span>
            </span>
          </span>
        </div>
      {/if}
    {/each}
  </div>
  <p id="twin-keys" class="keys">Click a component to select it. Space or Ctrl-click adds or removes it; Shift adds a range. {keyHelp()}</p>
</section>

<style>
  .twin {
    display: grid;
    gap: var(--space-2);
    align-content: start;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
  }

  .tools {
    display: flex;
    gap: var(--space-1);
  }

  .count,
  .keys,
  .empty {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .count {
    color: var(--text);
  }

  .list {
    display: grid;
    gap: 2px;
  }

  .option {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding-block: var(--space-2);
    cursor: pointer;
  }

  .box {
    flex: none;
    display: inline-grid;
    place-items: center;
    width: var(--space-4);
    height: var(--space-4);
    margin-top: 2px;
    border: 1px solid var(--line-control);
    border-radius: var(--radius-xs);
    font-size: var(--font-xs);
    line-height: 1;
  }

  .box.checked {
    border-color: var(--accent);
    color: var(--accent);
  }

  .body {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 2px;
  }

  .line {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .about {
    display: grid;
    gap: 1px;
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .unwired {
    color: var(--warning);
  }

  .swatches {
    display: flex;
    gap: 1px;
    margin-top: var(--space-1);
    forced-color-adjust: none;
  }

  .swatch {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }

  @media (forced-colors: active) {
    .box.checked {
      border-color: Highlight;
      color: CanvasText;
    }
  }
</style>
