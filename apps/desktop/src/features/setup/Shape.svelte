<script lang="ts">
  import type { Component } from "../../lib/api/bindings/Component";
  import type { Placement } from "../../lib/api/bindings/Placement";
  import type { Rgb } from "../../lib/api/bindings/Rgb";
  import { localLeds, outlineBox, viewLeds, type Point } from "./geometry";

  const LABEL_GAP = 22;
  const ARROW = 22;

  let {
    component,
    placement,
    colors,
    wired,
    selected,
    offset = { x: 0, y: 0 },
  }: {
    component: Component;
    placement: Placement;
    colors: Rgb[] | null;
    wired: boolean;
    selected: boolean;
    offset?: Point;
  } = $props();

  let at = $derived({ ...placement, x: placement.x + offset.x, y: placement.y + offset.y });
  let points = $derived(viewLeds(component.geometry, at));
  let box = $derived.by(() => {
    const outline = outlineBox(component.geometry);
    return "Strip" in component.geometry ? { width: outline.width, height: (radius * 2 + 10) / at.size } : outline;
  });
  let gap = $derived(
    points.length > 1 ? Math.min(...points.slice(1).map((point, index) => Math.hypot(point.x - points[index]!.x, point.y - points[index]!.y))) : at.size / 3,
  );
  let radius = $derived(Math.min(Math.max(gap * 0.36, 2.5), 9));
  let bounds = $derived.by(() => {
    const xs = points.map((point) => point.x);
    const ys = points.map((point) => point.y);
    const half = { x: (box.width * at.size) / 2, y: (box.height * at.size) / 2 };
    return {
      left: Math.min(...xs, at.x - half.x) - radius,
      right: Math.max(...xs, at.x + half.x) + radius,
      top: Math.min(...ys, at.y - half.y) - radius,
      bottom: Math.max(...ys, at.y + half.y) + radius,
    };
  });
  let pad = $derived(6 / Math.max(at.size, 1));
  let first = $derived(points[0] ?? null);
  let arrow = $derived.by(() => {
    const [start, next] = [points[0], points[1]];
    if (!start || !next) {
      return null;
    }
    const length = Math.hypot(next.x - start.x, next.y - start.y) || 1;
    const along = { x: (next.x - start.x) / length, y: (next.y - start.y) / length };
    const away = { x: start.x - at.x, y: start.y - at.y };
    const awayLength = Math.hypot(away.x, away.y) || 1;
    const out = { x: (away.x / awayLength) * (radius + 9), y: (away.y / awayLength) * (radius + 9) };
    const from = { x: start.x + out.x, y: start.y + out.y };
    return { from, to: { x: from.x + along.x * ARROW, y: from.y + along.y * ARROW } };
  });
  let numberAt = $derived.by(() => {
    if (!first) {
      return null;
    }
    const away = { x: first.x - at.x, y: first.y - at.y };
    const length = Math.hypot(away.x, away.y);
    if (length < 1) {
      return { x: first.x, y: first.y - radius - 8 };
    }
    const back = arrow ? { x: (arrow.from.x - arrow.to.x) / ARROW, y: (arrow.from.y - arrow.to.y) / ARROW } : { x: 0, y: 0 };
    return {
      x: first.x + (away.x / length) * (radius + 10) + back.x * 10,
      y: first.y + (away.y / length) * (radius + 10) + back.y * 10,
    };
  });
  let line = $derived("Line" in component.geometry ? lineOutline(component.geometry.Line.leds) : null);
  let label = $derived(component.name.length > 26 ? `${component.name.slice(0, 25)}…` : component.name);

  function lineOutline(count: number): string | null {
    const local = localLeds(component.geometry);
    return count > 0 && local.length > 1 ? local.map((point) => `${point.x},${point.y}`).join(" ") : null;
  }

  function fill(index: number): string {
    const color = colors?.[index];
    return color ? `rgb(${color.r} ${color.g} ${color.b})` : "var(--led-off)";
  }
</script>

<g class="shape" class:selected class:unwired={!wired} data-component={component.id}>
  <g transform="translate({at.x} {at.y}) rotate({at.rotation}) scale({at.flipped ? -at.size : at.size} {at.size})">
    {#if "Ring" in component.geometry}
      <circle class="outline" r="0.58" />
      <circle class="hub" r="0.2" />
    {:else if "Point" in component.geometry}
      <circle class="outline" r="0.25" />
    {:else if line}
      <polyline class="path" points={line} />
    {:else}
      <rect class="outline" x={-box.width / 2 - pad} y={-box.height / 2 - pad} width={box.width + 2 * pad} height={box.height + 2 * pad} rx={pad} />
      {#if "Perimeter" in component.geometry}
        <rect class="inner" x={-box.width / 2 + 0.08} y={-box.height / 2 + 0.08} width={Math.max(box.width - 0.16, 0.01)} height={Math.max(box.height - 0.16, 0.01)} />
      {/if}
    {/if}
  </g>
  {#each points as point, index (index)}
    <circle class="led" class:first={index === 0} cx={point.x} cy={point.y} r={index === 0 ? radius + 1.5 : radius} fill={wired ? fill(index) : "none"} />
  {/each}
  {#if numberAt}
    <text class="one" x={numberAt.x} y={numberAt.y} text-anchor="middle" dominant-baseline="central">1</text>
  {/if}
  {#if arrow}
    <line class="arrow" x1={arrow.from.x} y1={arrow.from.y} x2={arrow.to.x} y2={arrow.to.y} marker-end="url(#setup-arrow)" />
  {/if}
  {#if selected}
    <rect class="selection" x={bounds.left - 8} y={bounds.top - 8} width={bounds.right - bounds.left + 16} height={bounds.bottom - bounds.top + 16} rx="6" />
  {/if}
  <text class="name" x={at.x} y={bounds.bottom + LABEL_GAP} text-anchor="middle">{label}</text>
</g>

<style>
  .shape {
    cursor: grab;
  }

  .outline,
  .inner,
  .hub,
  .path {
    fill: var(--shape-fill);
    stroke: var(--line-control);
    stroke-width: 1.5;
    vector-effect: non-scaling-stroke;
  }

  .hub,
  .inner {
    fill: none;
    stroke: var(--line-subtle);
  }

  .path {
    fill: none;
    stroke-width: 6;
    stroke: var(--line-subtle);
  }

  .led {
    stroke: var(--surface-0);
    stroke-width: 1;
  }

  .led.first {
    stroke: var(--text-strong);
    stroke-width: 2;
  }

  .unwired .led {
    stroke: var(--text-faint);
    stroke-dasharray: 2 2;
  }

  .one {
    fill: var(--text-strong);
    font-size: 13px;
    font-weight: var(--weight-heading);
  }

  .arrow {
    stroke: var(--text);
    stroke-width: 2;
  }

  .name {
    fill: var(--text-dim);
    font-size: 15px;
  }

  .selected .name {
    fill: var(--text-strong);
    font-weight: var(--weight-strong);
  }

  .selection {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    stroke-dasharray: 6 4;
  }

  @media (forced-colors: active) {
    .outline,
    .inner,
    .hub,
    .path {
      stroke: CanvasText;
    }

    .selection {
      stroke: Highlight;
    }

    .one,
    .name {
      fill: CanvasText;
    }

    .arrow {
      stroke: CanvasText;
    }
  }
</style>
