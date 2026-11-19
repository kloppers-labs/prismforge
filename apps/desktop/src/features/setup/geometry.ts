import type { Corner } from "../../lib/api/bindings/Corner";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { GridPoint } from "../../lib/api/bindings/GridPoint";
import type { Placement } from "../../lib/api/bindings/Placement";

export type Point = { x: number; y: number };

export type Kind = "Strip" | "Ring" | "Matrix" | "Perimeter" | "Point" | "Line";

export const COMPONENT_LED_LIMIT = 4096;
export const LINE_POINT_LIMIT = 256;

export function kindOf(geometry: Geometry): Kind {
  return Object.keys(geometry)[0] as Kind;
}

export function ledCount(geometry: Geometry): number {
  if ("Strip" in geometry) {
    return geometry.Strip.leds;
  }
  if ("Ring" in geometry) {
    return geometry.Ring.leds;
  }
  if ("Line" in geometry) {
    return geometry.Line.leds;
  }
  if ("Matrix" in geometry) {
    return geometry.Matrix.columns * geometry.Matrix.rows;
  }
  if ("Perimeter" in geometry) {
    const { top, right, bottom, left } = geometry.Perimeter;
    return top + right + bottom + left;
  }
  return 1;
}

export function isValidShape(geometry: Geometry): boolean {
  const count = ledCount(geometry);
  const shape =
    "Ring" in geometry
      ? geometry.Ring.first_angle < 360
      : "Perimeter" in geometry
        ? geometry.Perimeter.width > 0 && geometry.Perimeter.height > 0
        : "Line" in geometry
          ? geometry.Line.points.length >= 2 && geometry.Line.points.length <= LINE_POINT_LIMIT
          : true;
  return shape && count >= 1 && count <= COMPONENT_LED_LIMIT;
}

function towards(from: Point, to: Point, amount: number): Point {
  return { x: from.x + (to.x - from.x) * amount, y: from.y + (to.y - from.y) * amount };
}

function spread(from: Point, to: Point, count: number): Point[] {
  return Array.from({ length: count }, (_, index) => towards(from, to, (index + 0.5) / count));
}

function corners(width: number, height: number): Point[] {
  const longer = Math.max(width, height);
  const [w, h] = [width / longer / 2, height / longer / 2];
  return [
    { x: -w, y: -h },
    { x: w, y: -h },
    { x: w, y: h },
    { x: -w, y: h },
  ];
}

const CORNER_INDEX: Record<Corner, number> = { TopLeft: 0, TopRight: 1, BottomRight: 2, BottomLeft: 3 };

function normalised(points: GridPoint[]): Point[] {
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  const [lowX, highX, lowY, highY] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const longer = Math.max(highX - lowX, highY - lowY);
  const centre = { x: (lowX + highX) / 2, y: (lowY + highY) / 2 };
  return points.map((point) => (longer > 0 ? { x: (point.x - centre.x) / longer, y: (point.y - centre.y) / longer } : { x: 0, y: 0 }));
}

function lineLeds(leds: number, points: GridPoint[], atEnd: boolean): Point[] {
  const path = atEnd ? normalised(points).reverse() : normalised(points);
  const lengths = path.slice(1).map((point, index) => Math.hypot(point.x - path[index]!.x, point.y - path[index]!.y));
  const total = lengths.reduce((sum, length) => sum + length, 0);
  return Array.from({ length: leds }, (_, index) => {
    let along = ((index + 0.5) / leds) * total;
    for (const [segment, length] of lengths.entries()) {
      if (along <= length || segment === lengths.length - 1) {
        const amount = length > 0 ? along / length : 0;
        return towards(path[segment]!, path[segment + 1]!, Math.min(amount, 1));
      }
      along -= length;
    }
    return path[0]!;
  });
}

/** Each LED's place in the component's own space in LED order, as the Rust model computes it; a test compares the two. */
export function localLeds(geometry: Geometry): Point[] {
  if (!isValidShape(geometry)) {
    return [];
  }
  if ("Strip" in geometry) {
    const [start, end] = [
      { x: -0.5, y: 0 },
      { x: 0.5, y: 0 },
    ];
    return geometry.Strip.first === "AtStart" ? spread(start, end, geometry.Strip.leds) : spread(end, start, geometry.Strip.leds);
  }
  if ("Ring" in geometry) {
    const { leds, first_angle, turn } = geometry.Ring;
    const sign = turn === "Clockwise" ? 1 : -1;
    return Array.from({ length: leds }, (_, index) => {
      const turns = first_angle / 360 + (sign * index) / leds;
      const angle = turns * 2 * Math.PI;
      return { x: 0.5 * Math.sin(angle), y: -0.5 * Math.cos(angle) };
    });
  }
  if ("Matrix" in geometry) {
    const { columns, rows, first, runs, serpentine } = geometry.Matrix;
    const pitch = 1 / Math.max(columns, rows);
    const [lines, perLine] = runs === "AlongRows" ? [rows, columns] : [columns, rows];
    const fromRight = first === "TopRight" || first === "BottomRight";
    const fromBottom = first === "BottomLeft" || first === "BottomRight";
    const leds: Point[] = [];
    for (let line = 0; line < lines; line += 1) {
      for (let step = 0; step < perLine; step += 1) {
        const along = serpentine && line % 2 === 1 ? perLine - 1 - step : step;
        const [column, row] = runs === "AlongRows" ? [along, line] : [line, along];
        const x = fromRight ? columns - 1 - column : column;
        const y = fromBottom ? rows - 1 - row : row;
        leds.push({ x: (x + 0.5 - columns / 2) * pitch, y: (y + 0.5 - rows / 2) * pitch });
      }
    }
    return leds;
  }
  if ("Perimeter" in geometry) {
    const { top, right, bottom, left, width, height, first, turn } = geometry.Perimeter;
    const box = corners(width, height);
    const sides = [top, right, bottom, left];
    const start = CORNER_INDEX[first];
    return [0, 1, 2, 3].flatMap((step) => {
      if (turn === "Clockwise") {
        const side = (start + step) % 4;
        return spread(box[side]!, box[(side + 1) % 4]!, sides[side]!);
      }
      const side = (start + 3 - step) % 4;
      return spread(box[(side + 1) % 4]!, box[side]!, sides[side]!);
    });
  }
  if ("Line" in geometry) {
    return lineLeds(geometry.Line.leds, geometry.Line.points, geometry.Line.first === "AtEnd");
  }
  return [{ x: 0, y: 0 }];
}

/** Where a point of the component's own space lands in the view: flip, scale, turn clockwise, move. */
export function locate(placement: Placement, point: Point): Point {
  const x = (placement.flipped ? -point.x : point.x) * placement.size;
  const y = point.y * placement.size;
  const angle = (placement.rotation * Math.PI) / 180;
  const [sin, cos] = [Math.sin(angle), Math.cos(angle)];
  return { x: placement.x + x * cos - y * sin, y: placement.y + x * sin + y * cos };
}

export function viewLeds(geometry: Geometry, placement: Placement): Point[] {
  return localLeds(geometry).map((point) => locate(placement, point));
}

/** The outline's width and height in the component's own space; the longer side is 1. */
export function outlineBox(geometry: Geometry): { width: number; height: number } {
  if ("Strip" in geometry) {
    return { width: 1, height: Math.max(0.08, Math.min(0.25, 1.6 / Math.max(geometry.Strip.leds, 1))) };
  }
  if ("Matrix" in geometry) {
    const longer = Math.max(geometry.Matrix.columns, geometry.Matrix.rows);
    return { width: geometry.Matrix.columns / longer, height: geometry.Matrix.rows / longer };
  }
  if ("Perimeter" in geometry) {
    const longer = Math.max(geometry.Perimeter.width, geometry.Perimeter.height, 1);
    return { width: geometry.Perimeter.width / longer, height: geometry.Perimeter.height / longer };
  }
  if ("Point" in geometry) {
    return { width: 0.3, height: 0.3 };
  }
  return { width: 1, height: 1 };
}

export function withLeds(geometry: Geometry, leds: number): Geometry {
  if ("Strip" in geometry) {
    return { Strip: { ...geometry.Strip, leds } };
  }
  if ("Ring" in geometry) {
    return { Ring: { ...geometry.Ring, leds } };
  }
  if ("Line" in geometry) {
    return { Line: { ...geometry.Line, leds } };
  }
  return geometry;
}

/** The other running order of the same LEDs; the picture's flip is a separate setting. */
export function reversed(geometry: Geometry): Geometry {
  if ("Strip" in geometry) {
    return { Strip: { ...geometry.Strip, first: geometry.Strip.first === "AtStart" ? "AtEnd" : "AtStart" } };
  }
  if ("Line" in geometry) {
    return { Line: { ...geometry.Line, first: geometry.Line.first === "AtStart" ? "AtEnd" : "AtStart" } };
  }
  if ("Ring" in geometry) {
    return { Ring: { ...geometry.Ring, turn: geometry.Ring.turn === "Clockwise" ? "CounterClockwise" : "Clockwise" } };
  }
  if ("Perimeter" in geometry) {
    return { Perimeter: { ...geometry.Perimeter, turn: geometry.Perimeter.turn === "Clockwise" ? "CounterClockwise" : "Clockwise" } };
  }
  if ("Matrix" in geometry) {
    return { Matrix: { ...geometry.Matrix, first: lastCorner(geometry) } };
  }
  return geometry;
}

/** The corner the last LED of a matrix sits in; starting there with the same runs gives the LEDs in reverse. */
function lastCorner(geometry: Extract<Geometry, { Matrix: unknown }>): Corner {
  const last = localLeds(geometry).at(-1) ?? { x: 0, y: 0 };
  if (last.y < 0) {
    return last.x < 0 ? "TopLeft" : "TopRight";
  }
  return last.x < 0 ? "BottomLeft" : "BottomRight";
}

export const POINT_RANGE = { min: -1000, max: 1000 } as const;

function onLine(geometry: Geometry, change: (points: GridPoint[]) => GridPoint[]): Geometry {
  return "Line" in geometry ? { Line: { ...geometry.Line, points: change(geometry.Line.points) } } : geometry;
}

function inRange(value: number): number {
  return Math.min(Math.max(Math.round(value), POINT_RANGE.min), POINT_RANGE.max);
}

/** Adds a point after the last one, continuing its last segment. */
export function addPoint(geometry: Geometry): Geometry {
  return onLine(geometry, (points) => {
    if (points.length >= LINE_POINT_LIMIT) {
      return points;
    }
    const last = points.at(-1) ?? { x: 0, y: 0 };
    const before = points.at(-2) ?? { x: last.x - 100, y: last.y };
    return [...points, { x: inRange(2 * last.x - before.x), y: inRange(2 * last.y - before.y) }];
  });
}

export function movePoint(geometry: Geometry, index: number, to: { x: number; y: number }): Geometry {
  return onLine(geometry, (points) => points.map((point, at) => (at === index ? { ...point, x: inRange(to.x), y: inRange(to.y) } : point)));
}

export function removePoint(geometry: Geometry, index: number): Geometry {
  return onLine(geometry, (points) => (points.length <= 2 ? points : points.filter((_, at) => at !== index)));
}
