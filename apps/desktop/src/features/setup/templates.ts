import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { VIEWS } from "./edits";

/** A proposed component: a name, a shape and a place, wired to nothing until the user chooses its output. */
export type Slot = { name: string; geometry: Geometry; x: number; y: number; size: number; rotation: number };

export type Template = { id: "tower" | "desk"; label: string; view: ViewId; slots: Slot[] };

const ring = (leds: number): Geometry => ({ Ring: { leds, first_angle: 0, turn: "Clockwise" } });
const strip = (leds: number): Geometry => ({ Strip: { leds, first: "AtStart" } });

export const TEMPLATES: Template[] = [
  {
    id: "tower",
    label: "Tower PC",
    view: VIEWS.pc,
    slots: [
      { name: "Front fan 1", geometry: ring(8), x: 160, y: 130, size: 100, rotation: 0 },
      { name: "Front fan 2", geometry: ring(8), x: 160, y: 300, size: 100, rotation: 0 },
      { name: "Front fan 3", geometry: ring(8), x: 160, y: 470, size: 100, rotation: 0 },
      { name: "Rear fan", geometry: ring(8), x: 910, y: 130, size: 100, rotation: 0 },
      { name: "Cooler strip 1", geometry: strip(8), x: 440, y: 270, size: 120, rotation: 90 },
      { name: "Cooler strip 2", geometry: strip(8), x: 560, y: 270, size: 120, rotation: 90 },
      { name: "Memory module 1", geometry: strip(8), x: 690, y: 270, size: 130, rotation: 90 },
      { name: "Memory module 2", geometry: strip(8), x: 800, y: 270, size: 130, rotation: 90 },
      { name: "Graphics card", geometry: strip(12), x: 560, y: 470, size: 320, rotation: 0 },
      { name: "Board edge", geometry: strip(6), x: 530, y: 610, size: 200, rotation: 0 },
    ],
  },
  {
    id: "desk",
    label: "Desk",
    view: VIEWS.desk,
    slots: [
      { name: "Monitor edge", geometry: { Perimeter: { top: 12, right: 8, bottom: 12, left: 8, width: 16, height: 9, first: "BottomLeft", turn: "Clockwise" } }, x: 500, y: 220, size: 420, rotation: 0 },
      { name: "Desk edge strip", geometry: strip(30), x: 500, y: 640, size: 860, rotation: 0 },
      { name: "Keyboard", geometry: { Matrix: { columns: 12, rows: 4, first: "TopLeft", runs: "AlongRows", serpentine: false } }, x: 430, y: 500, size: 340, rotation: 0 },
      { name: "Light bar", geometry: strip(10), x: 860, y: 220, size: 220, rotation: 90 },
      { name: "Mouse logo", geometry: { Point: {} }, x: 700, y: 520, size: 40, rotation: 0 },
    ],
  },
];
