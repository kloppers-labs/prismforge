import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { LiveOutput } from "../../lib/api/bindings/LiveOutput";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { placementOf } from "./edits";
import { parts, sameTarget, targetOf } from "./wiring";

/** Each component's colours, cut from its output's LEDs by its part; mirrored branches read the same LEDs. */
export function componentColors(setup: Setup, colorsOf: (target: OutputTarget) => Rgb[] | null): Record<ComponentId, Rgb[]> {
  const result: Record<ComponentId, Rgb[]> = {};
  for (const output of setup.outputs) {
    const colors = colorsOf(targetOf(output));
    if (!colors) {
      continue;
    }
    for (const part of parts(setup, output)) {
      result[part.component] = colors.slice(part.start, part.end);
    }
  }
  return result;
}

/** The LEDs of a device or zone in a live frame; a whole device is its zones in order. */
export function liveColors(live: LiveOutput, target: OutputTarget): Rgb[] | null {
  const device = live.devices.find((each) => each.device === target.device);
  if (!device) {
    return null;
  }
  if (target.zone === null) {
    return device.zones.flatMap((zone) => zone.colors);
  }
  return device.zones.find((zone) => zone.zone === target.zone)?.colors ?? null;
}

export function viewTargets(setup: Setup, view: ViewId): OutputTarget[] {
  const targets: OutputTarget[] = [];
  for (const output of setup.outputs) {
    const placed = output.branches.flat().some((id) => placementOf(setup, view, id) !== null);
    const target = targetOf(output);
    if (placed && !targets.some((each) => sameTarget(each, target))) {
      targets.push(target);
    }
  }
  return targets;
}
