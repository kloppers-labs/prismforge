<script lang="ts">
  import type { ComponentId } from "../../lib/api/bindings/ComponentId";
  import type { Rgb } from "../../lib/api/bindings/Rgb";
  import { CANVAS } from "./edits";
  import { viewLeds, type Point } from "./geometry";
  import type { EditAction } from "./keys";
  import Shape from "./Shape.svelte";
  import type { SetupState } from "./state.svelte";
  import { outputOf } from "./wiring";

  const GRID = 50;
  const DRAG_START = 3;

  let {
    setup,
    colors,
    live,
    onaction,
  }: {
    setup: SetupState;
    colors: Record<ComponentId, Rgb[]> | null;
    live: boolean;
    onaction: (action: EditAction) => void;
  } = $props();

  let svg: SVGSVGElement | undefined = $state();
  let drag = $state<{ id: ComponentId; from: Point; moved: Point; pointer: number; started: boolean } | null>(null);
  let draft = $derived(setup.draft);
  let view = $derived(setup.view);
  let placements = $derived(view?.placements ?? []);
  let components = $derived(new Map(draft.components.map((component) => [component.id, component])));
  let centres = $derived(new Map(placements.map((placement) => [placement.component, { x: placement.x, y: placement.y }])));
  let mirrors = $derived(
    draft.outputs
      .filter((output) => output.branches.length > 1)
      .map((output) => output.branches.flatMap((branch) => branch.flatMap((id) => centres.get(id) ?? [])))
      .filter((points) => points.length > 1),
  );
  let chains = $derived(
    draft.outputs.flatMap((output) =>
      output.branches.flatMap((branch) =>
        branch.slice(1).flatMap((id, index) => {
          const before = ends(branch[index]!);
          const after = ends(id);
          return before && after ? [{ from: before.last, to: after.first }] : [];
        }),
      ),
    ),
  );
  let title = $derived(`Drawing of the ${view?.name ?? ""} view`);

  function ends(id: ComponentId): { first: Point; last: Point } | null {
    const component = components.get(id);
    const placement = placements.find((each) => each.component === id);
    if (!component || !placement) {
      return null;
    }
    const points = viewLeds(component.geometry, placement);
    return points.length > 0 ? { first: points[0]!, last: points.at(-1)! } : null;
  }

  function toView(event: PointerEvent): Point | null {
    const matrix = svg?.getScreenCTM?.();
    if (!svg || !matrix) {
      return null;
    }
    const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    return { x: point.x, y: point.y };
  }

  function offsetFor(id: ComponentId): Point {
    return drag?.started && setup.selection.includes(id) ? drag.moved : { x: 0, y: 0 };
  }

  function onpointerdown(event: PointerEvent) {
    const shape = (event.target as Element | null)?.closest<SVGElement>("[data-component]");
    if (!shape || event.button !== 0) {
      if (!shape && event.button === 0 && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
        setup.select([]);
      }
      return;
    }
    const id = shape.dataset.component!;
    if (event.ctrlKey || event.metaKey) {
      setup.toggle(id);
      return;
    }
    if (event.shiftKey) {
      setup.select([...setup.selection, id]);
    } else if (!setup.selection.includes(id)) {
      setup.select([id]);
    }
    const from = toView(event);
    if (from) {
      drag = { id, from, moved: { x: 0, y: 0 }, pointer: event.pointerId, started: false };
      svg?.setPointerCapture?.(event.pointerId);
    }
  }

  function onpointermove(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointer) {
      return;
    }
    const at = toView(event);
    if (!at) {
      return;
    }
    const moved = { x: Math.round(at.x - drag.from.x), y: Math.round(at.y - drag.from.y) };
    drag = { ...drag, moved, started: drag.started || Math.hypot(moved.x, moved.y) >= DRAG_START };
  }

  function onpointerup(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointer) {
      return;
    }
    const done = drag;
    drag = null;
    if (done.started) {
      onaction({ kind: "move", dx: done.moved.x, dy: done.moved.y });
    }
  }

  function oncontextmenu(event: MouseEvent) {
    const id = (event.target as Element | null)?.closest<SVGElement>("[data-component]")?.dataset.component;
    const row = id ? document.querySelector<HTMLElement>(`[data-row="${CSS.escape(id)}"]`) : null;
    if (!id || !row) {
      return;
    }
    event.preventDefault();
    if (!setup.selection.includes(id)) {
      setup.select([id]);
    }
    row.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: event.clientX, clientY: event.clientY }));
  }

  function onkeydown(event: KeyboardEvent) {
    if (drag && event.key === "Escape") {
      event.preventDefault();
      drag = null;
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="canvas" class:live role="img" aria-label={title} aria-describedby="drawing-keys">
  <svg
    bind:this={svg}
    viewBox="0 0 {CANVAS.width} {CANVAS.height}"
    aria-hidden="true"
    focusable="false"
    {onpointerdown}
    {onpointermove}
    {onpointerup}
    {oncontextmenu}
    onpointercancel={() => (drag = null)}
  >
    <defs>
      <pattern id="setup-grid" width={GRID} height={GRID} patternUnits="userSpaceOnUse">
        <path d="M {GRID} 0 L 0 0 0 {GRID}" class="grid" />
      </pattern>
      <marker id="setup-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 0 L 10 5 L 0 10 z" class="head" />
      </marker>
    </defs>
    <rect class="paper" width={CANVAS.width} height={CANVAS.height} />
    <rect width={CANVAS.width} height={CANVAS.height} fill="url(#setup-grid)" />
    {#each mirrors as points, index (index)}
      <polyline class="mirror" points={points.map((point) => `${point.x},${point.y}`).join(" ")} />
    {/each}
    {#each chains as link, index (index)}
      <line class="chain" x1={link.from.x} y1={link.from.y} x2={link.to.x} y2={link.to.y} marker-end="url(#setup-arrow)" />
    {/each}
    {#each placements as placement (placement.component)}
      {@const component = components.get(placement.component)}
      {#if component}
        <Shape
          {component}
          {placement}
          colors={colors?.[component.id] ?? null}
          wired={outputOf(draft, component.id) !== null}
          selected={setup.selection.includes(component.id)}
          offset={offsetFor(component.id)}
        />
      {/if}
    {/each}
  </svg>
</div>
<p id="drawing-keys" class="keys">Click a component to select it and drag to move it; Escape cancels a drag. With the keyboard, use the list beside the drawing: it has every component and every action.</p>

<style>
  .canvas {
    border: 1px dashed var(--line-control);
    border-radius: var(--radius-m);
    background: var(--surface-stage);
    overflow: hidden;
    touch-action: none;
  }

  .canvas.live {
    border: 2px solid var(--ok);
  }

  svg {
    display: block;
    width: 100%;
    height: auto;
    forced-color-adjust: none;
    user-select: none;
  }

  .paper {
    fill: var(--surface-stage);
  }

  .grid {
    fill: none;
    stroke: var(--canvas-grid);
    stroke-width: 1;
  }

  .mirror {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.5;
    stroke-dasharray: 3 6;
    opacity: 0.7;
  }

  .chain {
    stroke: var(--text-dim);
    stroke-width: 1.5;
  }

  .head {
    fill: var(--text);
  }

  .keys {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
