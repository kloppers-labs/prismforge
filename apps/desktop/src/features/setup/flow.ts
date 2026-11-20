import type { Flow } from "../../lib/api/bindings/Flow";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { VIEW_NAMES } from "./edits";

const NAMED: Record<number, string> = { 0: "left to right", 90: "top to bottom", 180: "right to left", 270: "bottom to top" };

export const START_FLOW: Flow = { Along: { angle: 0 } };

export function flowText(flow: Flow): string {
  if ("Along" in flow) {
    const angle = flow.Along.angle;
    return NAMED[angle] ? `flowing ${NAMED[angle]} (${angle}°)` : `flowing at ${angle}° clockwise from left to right`;
  }
  return `out from a centre ${flow.FromCentre.x}% across and ${flow.FromCentre.y}% down`;
}

/** The apply mode of a running picture, with its flow, as the Effects screen names it. */
export function acrossText(view: ViewId, flow: Flow): string {
  return `Runs across the setup: ${VIEW_NAMES[view] ?? view} view, ${flowText(flow)}`;
}
