import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { MatrixSize } from "../../lib/api/bindings/MatrixSize";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { CANVAS } from "./edits";
import { COMPONENT_LED_LIMIT } from "./geometry";

/** The generic shapes PrismForge draws; each is a plain outline made for PrismForge, not a product picture. */
export type Shape = "Strip" | "Ring" | "DualRing" | "Matrix" | "Perimeter" | "Point";

export const SHAPES: { value: Shape; label: string; size: number }[] = [
  { value: "Strip", label: "Strip or bar", size: 300 },
  { value: "Ring", label: "Ring fan", size: 110 },
  { value: "DualRing", label: "Fan with two rings", size: 110 },
  { value: "Matrix", label: "Matrix, such as a keyboard or a panel", size: 360 },
  { value: "Perimeter", label: "Edge of a screen", size: 400 },
  { value: "Point", label: "Single light, such as a logo", size: 40 },
];

/** One part of a shape: a dual-ring fan is two rings chained on one output, the inner one smaller at the same centre. */
export type ShapePart = { suffix: string; geometry: Geometry; scale: number };

function limited(leds: number): number {
  return Math.min(Math.max(Math.round(leds), 1), COMPONENT_LED_LIMIT);
}

export function shapeParts(shape: Shape, count: number, matrix: MatrixSize | null = null): ShapePart[] {
  const leds = limited(count);
  switch (shape) {
    case "Strip":
      return [{ suffix: "", geometry: { Strip: { leds, first: "AtStart" } }, scale: 1 }];
    case "Ring":
      return [{ suffix: "", geometry: { Ring: { leds, first_angle: 0, turn: "Clockwise" } }, scale: 1 }];
    case "DualRing": {
      const outer = Math.max(1, Math.ceil(leds * 0.6));
      const inner = Math.max(1, leds - outer);
      return [
        { suffix: " outer ring", geometry: { Ring: { leds: outer, first_angle: 0, turn: "Clockwise" } }, scale: 1 },
        { suffix: " inner ring", geometry: { Ring: { leds: inner, first_angle: 0, turn: "Clockwise" } }, scale: 0.55 },
      ];
    }
    case "Matrix": {
      const [columns, rows] = matrix && matrix.width * matrix.height === leds ? [matrix.width, matrix.height] : [leds, 1];
      return [{ suffix: "", geometry: { Matrix: { columns, rows, first: "TopLeft", runs: "AlongRows", serpentine: false } }, scale: 1 }];
    }
    case "Perimeter": {
      const across = Math.max(1, Math.floor(leds * 0.32));
      const side = Math.floor((leds - 2 * across) / 2);
      const top = leds - 2 * side - across;
      return [{ suffix: "", geometry: { Perimeter: { top, right: side, bottom: across, left: side, width: 16, height: 9, first: "TopLeft", turn: "Clockwise" } }, scale: 1 }];
    }
    case "Point":
      return [{ suffix: "", geometry: { Point: {} }, scale: 1 }];
  }
}

export function sizeOf(shape: Shape): number {
  return SHAPES.find((each) => each.value === shape)?.size ?? 120;
}

/** The first spot on a coarse grid that no placed component sits close to; the middle when the view is full. */
export function freeSpot(setup: Setup, view: ViewId): { x: number; y: number } {
  const taken = setup.views.find((each) => each.id === view)?.placements ?? [];
  for (let y = 120; y <= CANVAS.height - 100; y += 130) {
    for (let x = 140; x <= CANVAS.width - 120; x += 150) {
      if (taken.every((placement) => Math.hypot(placement.x - x, placement.y - y) > 100)) {
        return { x, y };
      }
    }
  }
  return { x: CANVAS.width / 2, y: CANVAS.height / 2 };
}
