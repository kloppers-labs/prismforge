import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { MatrixSize } from "../../lib/api/bindings/MatrixSize";
import type { OutlineKind } from "../../lib/api/bindings/OutlineKind";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { CANVAS } from "./edits";
import { COMPONENT_LED_LIMIT } from "./geometry";

/** The generic shapes PrismForge draws; each is a plain outline made for PrismForge, not a product picture. */
export type Shape = "Strip" | "Ring" | "DualRing" | "Matrix" | "Perimeter" | "Point" | "Line" | "BlockStrip" | "BlockRing" | "BlockMatrix";

export const SHAPES: { value: Shape; label: string; size: number }[] = [
  { value: "Strip", label: "Strip or bar", size: 300 },
  { value: "Ring", label: "Ring fan", size: 110 },
  { value: "DualRing", label: "Fan with two rings", size: 110 },
  { value: "Matrix", label: "Matrix, such as a keyboard or a panel", size: 360 },
  { value: "Perimeter", label: "Edge of a screen", size: 400 },
  { value: "Point", label: "Single light, such as a logo", size: 40 },
  { value: "Line", label: "Free line, such as a strip round a corner", size: 300 },
  { value: "BlockStrip", label: "Block with a strip of LEDs, such as a graphics card", size: 300 },
  { value: "BlockRing", label: "Block with a ring of LEDs, such as a cooler plate", size: 110 },
  { value: "BlockMatrix", label: "Block with a grid of LEDs, such as a panel", size: 300 },
];

/** Unlit drawings to place components by; each is a plain outline made for PrismForge. */
export const OUTLINES: { kind: OutlineKind; name: string; x: number; y: number; width: number; height: number }[] = [
  { kind: "Case", name: "Case", x: 500, y: 350, width: 400, height: 560 },
  { kind: "Pc", name: "PC", x: 850, y: 420, width: 160, height: 280 },
  { kind: "Monitor", name: "Monitor", x: 500, y: 230, width: 480, height: 300 },
  { kind: "Desk", name: "Desk", x: 500, y: 600, width: 940, height: 60 },
];

/** One part of a shape: a dual-ring fan is two rings chained on one output, the inner one smaller at the same centre. */
export type ShapePart = { suffix: string; geometry: Geometry; scale: number; frame?: { width: number; height: number } };

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
    case "Line":
      return [{ suffix: "", geometry: { Line: { leds, points: [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 60 }], first: "AtStart" } }, scale: 1 }];
    case "BlockStrip":
      return [{ ...shapeParts("Strip", leds)[0]!, frame: { width: 112, height: 40 } }];
    case "BlockRing":
      return [{ ...shapeParts("Ring", leds)[0]!, frame: { width: 130, height: 130 } }];
    case "BlockMatrix":
      return [{ ...shapeParts("Matrix", leds, matrix)[0]!, frame: { width: 112, height: 112 } }];
  }
}

export function sizeOf(shape: Shape): number {
  return SHAPES.find((each) => each.value === shape)?.size ?? 120;
}

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
