import type { Component } from "../../lib/api/bindings/Component";
import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { DeviceRef } from "../../lib/api/bindings/DeviceRef";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { OutputId } from "../../lib/api/bindings/OutputId";
import type { Placement } from "../../lib/api/bindings/Placement";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupOutput } from "../../lib/api/bindings/SetupOutput";
import type { SetupView } from "../../lib/api/bindings/SetupView";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { outputFor, outputOf } from "./wiring";

export const CANVAS = { width: 1000, height: 700 } as const;
export const ROTATION_STEP = 15;
export const NUDGE = 10;
export const SIZE_LIMITS = { min: 10, max: 1000 } as const;
export const VIEWS = { pc: "v-pc", desk: "v-desk" } as const satisfies Record<string, ViewId>;
const VIEW_NAMES: Record<ViewId, string> = { [VIEWS.pc]: "PC", [VIEWS.desk]: "Desk" };

export type OutputChoice = { device: DeviceRef; zone: number | null; label: string; zoneName: string | null };

/** How a component joins an output: alone, beside the others with the same colours, or after the last part on the wire. */
export type Joining = "own" | "mirror" | "chain";

export type NewComponent = Pick<Component, "id" | "name" | "geometry">;

export function withViews(setup: Setup): Setup {
  const missing = [VIEWS.pc, VIEWS.desk].filter((id) => !setup.views.some((view) => view.id === id));
  if (missing.length === 0) {
    return setup;
  }
  return { ...setup, views: [...setup.views, ...missing.map((id) => ({ id, name: VIEW_NAMES[id]!, placements: [] }))] };
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(Math.max(Math.round(value), low), high);
}

function turn(degrees: number): number {
  return ((Math.round(degrees) % 360) + 360) % 360;
}

function inView(setup: Setup, view: ViewId, change: (view: SetupView) => SetupView): Setup {
  return { ...setup, views: setup.views.map((each) => (each.id === view ? change(each) : each)) };
}

function onPlacements(setup: Setup, view: ViewId, ids: ComponentId[], change: (placement: Placement) => Placement): Setup {
  return inView(setup, view, (each) => ({
    ...each,
    placements: each.placements.map((placement) => (ids.includes(placement.component) ? change(placement) : placement)),
  }));
}

function onComponent(setup: Setup, id: ComponentId, change: (component: Component) => Component): Setup {
  return { ...setup, components: setup.components.map((component) => (component.id === id ? change(component) : component)) };
}

function onOutput(setup: Setup, id: OutputId, change: (output: SetupOutput) => SetupOutput | null): Setup {
  return {
    ...setup,
    outputs: setup.outputs.flatMap((output) => {
      if (output.id !== id) {
        return [output];
      }
      const next = change(output);
      return next ? [next] : [];
    }),
  };
}

export function placementOf(setup: Setup, view: ViewId, id: ComponentId): Placement | null {
  return setup.views.find((each) => each.id === view)?.placements.find((placement) => placement.component === id) ?? null;
}

export function addComponent(setup: Setup, view: ViewId, component: NewComponent, at: { x: number; y: number; size?: number; rotation?: number }): Setup {
  const placed = withViews({ ...setup, components: [...setup.components, { ...component }] });
  return place(placed, view, component.id, at);
}

export function place(setup: Setup, view: ViewId, id: ComponentId, at: { x: number; y: number; size?: number; rotation?: number }): Setup {
  if (placementOf(setup, view, id)) {
    return setup;
  }
  const placement: Placement = {
    component: id,
    x: clamp(at.x, 0, CANVAS.width),
    y: clamp(at.y, 0, CANVAS.height),
    size: clamp(at.size ?? 120, SIZE_LIMITS.min, SIZE_LIMITS.max),
    rotation: turn(at.rotation ?? 0),
    flipped: false,
  };
  return inView(withViews(setup), view, (each) => ({ ...each, placements: [...each.placements, placement] }));
}

export function unplace(setup: Setup, view: ViewId, id: ComponentId): Setup {
  return inView(setup, view, (each) => ({ ...each, placements: each.placements.filter((placement) => placement.component !== id) }));
}

/** Moves every listed component by the same amount, so the group keeps its shape and stays inside the view. */
export function moveBy(setup: Setup, view: ViewId, ids: ComponentId[], dx: number, dy: number): Setup {
  const moving = ids.flatMap((id) => placementOf(setup, view, id) ?? []);
  if (moving.length === 0) {
    return setup;
  }
  const xs = moving.map((placement) => placement.x);
  const ys = moving.map((placement) => placement.y);
  const stepX = clamp(dx, -Math.min(...xs), CANVAS.width - Math.max(...xs));
  const stepY = clamp(dy, -Math.min(...ys), CANVAS.height - Math.max(...ys));
  return onPlacements(setup, view, ids, (placement) => ({ ...placement, x: placement.x + stepX, y: placement.y + stepY }));
}

export function moveTo(setup: Setup, view: ViewId, id: ComponentId, x: number, y: number): Setup {
  return onPlacements(setup, view, [id], (placement) => ({ ...placement, x: clamp(x, 0, CANVAS.width), y: clamp(y, 0, CANVAS.height) }));
}

export function rotateBy(setup: Setup, view: ViewId, ids: ComponentId[], steps: number): Setup {
  return onPlacements(setup, view, ids, (placement) => ({ ...placement, rotation: turn(placement.rotation + steps * ROTATION_STEP) }));
}

export function setRotation(setup: Setup, view: ViewId, id: ComponentId, degrees: number): Setup {
  return onPlacements(setup, view, [id], (placement) => ({ ...placement, rotation: turn(degrees) }));
}

export function flip(setup: Setup, view: ViewId, ids: ComponentId[]): Setup {
  return onPlacements(setup, view, ids, (placement) => ({ ...placement, flipped: !placement.flipped }));
}

export function setSize(setup: Setup, view: ViewId, id: ComponentId, size: number): Setup {
  return onPlacements(setup, view, [id], (placement) => ({ ...placement, size: clamp(size, SIZE_LIMITS.min, SIZE_LIMITS.max) }));
}

export function rename(setup: Setup, id: ComponentId, name: string): Setup {
  return onComponent(setup, id, (component) => ({ ...component, name }));
}

export function setGeometry(setup: Setup, id: ComponentId, geometry: Geometry): Setup {
  return onComponent(setup, id, (component) => ({ ...component, geometry }));
}

/** Wires a component that is on no output; `id` names the output when a new one is needed. */
export function wire(setup: Setup, component: ComponentId, choice: OutputChoice, joining: Joining, id: OutputId): Setup {
  const free = unwire(setup, component);
  const existing = outputFor(free, { device: choice.device, zone: choice.zone });
  if (existing && joining !== "own") {
    return onOutput(free, existing.id, (output) => ({
      ...output,
      branches:
        joining === "mirror"
          ? [...output.branches, [component]]
          : output.branches.map((branch, index) => (index === output.branches.length - 1 ? [...branch, component] : branch)),
    }));
  }
  if (existing) {
    return setup;
  }
  const output: SetupOutput = {
    id,
    device: choice.device,
    zone: choice.zone,
    label: choice.label,
    zone_name: choice.zoneName,
    first_led: 0,
    branches: [[component]],
  };
  return { ...free, outputs: [...free.outputs, output] };
}

export function unwire(setup: Setup, component: ComponentId): Setup {
  const output = outputOf(setup, component);
  if (!output) {
    return setup;
  }
  return onOutput(setup, output.id, (each) => {
    const branches = each.branches.map((branch) => branch.filter((id) => id !== component)).filter((branch) => branch.length > 0);
    return branches.length > 0 ? { ...each, branches } : null;
  });
}

export function moveInChain(setup: Setup, component: ComponentId, delta: number): Setup {
  const output = outputOf(setup, component);
  if (!output) {
    return setup;
  }
  return onOutput(setup, output.id, (each) => ({
    ...each,
    branches: each.branches.map((branch) => {
      const from = branch.indexOf(component);
      const to = from + delta;
      if (from < 0 || to < 0 || to >= branch.length) {
        return branch;
      }
      const next = [...branch];
      next.splice(from, 1);
      next.splice(to, 0, component);
      return next;
    }),
  }));
}

export function setFirstLed(setup: Setup, output: OutputId, first: number): Setup {
  return onOutput(setup, output, (each) => ({ ...each, first_led: Math.max(0, Math.round(first)) }));
}

export function removeComponent(setup: Setup, id: ComponentId): Setup {
  const free = unwire(setup, id);
  return {
    ...free,
    components: free.components.filter((component) => component.id !== id),
    views: free.views.map((view) => ({ ...view, placements: view.placements.filter((placement) => placement.component !== id) })),
  };
}
