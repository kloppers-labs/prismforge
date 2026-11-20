import type { Component } from "../../lib/api/bindings/Component";
import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { Placement } from "../../lib/api/bindings/Placement";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupOutput } from "../../lib/api/bindings/SetupOutput";
import type { SetupOutputStatus } from "../../lib/api/bindings/SetupOutputStatus";
import type { SetupProblem } from "../../lib/api/bindings/SetupProblem";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { nameProblemText } from "../../lib/errors";
import type { Names } from "../../lib/names";
import { capacity, deviceFor } from "./bounds";
import { placementOf } from "./edits";
import { COMPONENT_LED_LIMIT, kindOf, ledCount, viewLeds, type Point } from "./geometry";
import { chainOf, outputOf, parts, sharedWith, span } from "./wiring";

const COUNTS = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

export const KIND_NAMES = {
  Strip: "Strip",
  Ring: "Ring",
  Matrix: "Matrix",
  Perimeter: "Perimeter",
  Point: "Point",
  Line: "Free line",
} as const;

export function leds(count: number): string {
  return count === 1 ? "1 LED" : `${count} LEDs`;
}

export function countWord(count: number): string {
  return COUNTS[count] ?? String(count);
}

export function listText(items: string[]): string {
  if (items.length <= 1) {
    return items.join("");
  }
  return `${items.slice(0, -1).join(", ")} and ${items.at(-1)}`;
}

export function nameOf(setup: Setup, id: string): string {
  return (
    setup.components.find((component) => component.id === id)?.name ??
    setup.views.flatMap((view) => view.outlines).find((outline) => outline.id === id)?.name ??
    id
  );
}

export function selectionName(setup: Setup, ids: string[]): string {
  if (ids.length === 1) {
    return nameOf(setup, ids[0]!);
  }
  return ids.every((id) => setup.components.some((component) => component.id === id)) ? `${ids.length} components` : `${ids.length} items`;
}

export function kindText(geometry: Geometry, framed = false): string {
  const frame = framed ? ", in a frame" : "";
  if ("Matrix" in geometry) {
    return `Matrix of ${geometry.Matrix.columns} × ${geometry.Matrix.rows}, ${leds(ledCount(geometry))}${frame}`;
  }
  return `${KIND_NAMES[kindOf(geometry)]}, ${leds(ledCount(geometry))}${frame}`;
}

export function outlinePlaceText(outline: { x: number; y: number; width: number; height: number; rotation: number }): string {
  return `x ${outline.x}, y ${outline.y}, ${outline.width} × ${outline.height}, turned ${outline.rotation}°`;
}

export function placeText(placement: Placement): string {
  return `x ${placement.x}, y ${placement.y}, size ${placement.size}, turned ${placement.rotation}°${placement.flipped ? ", flipped" : ""}`;
}

const SIDES = ["right", "bottom right", "bottom", "bottom left", "left", "top left", "top", "top right"];
const WAYS = ["right", "down and right", "down", "down and left", "left", "up and left", "up", "up and right"];

function sector(from: Point, to: Point): number {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  return (Math.round(angle / (Math.PI / 4)) + 8) % 8;
}

function centreOf(points: Point[]): Point {
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  return { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: (Math.min(...ys) + Math.max(...ys)) / 2 };
}

/** Where LED 1 is and which way the LEDs run, as the drawing shows them after the turn and the flip. */
export function firstLedText(geometry: Geometry, placement: Placement): string {
  const points = viewLeds(geometry, placement);
  if (points.length <= 1) {
    return "One LED";
  }
  const centre = centreOf(points);
  const [first, second] = [points[0]!, points[1]!];
  const side = SIDES[sector(centre, first)]!;
  const corner = `${first.y < centre.y ? "top" : "bottom"} ${first.x < centre.x ? "left" : "right"}`;
  if ("Ring" in geometry || "Perimeter" in geometry) {
    const cross = (first.x - centre.x) * (second.y - centre.y) - (first.y - centre.y) * (second.x - centre.x);
    const where = "Ring" in geometry ? `at the ${side}` : `in the ${corner} corner`;
    return `LED 1 ${where}, runs ${cross > 0 ? "clockwise" : "counter-clockwise"}`;
  }
  if ("Matrix" in geometry) {
    const { runs, serpentine } = geometry.Matrix;
    return `LED 1 in the ${corner} corner, ${runs === "AlongRows" ? "row by row" : "column by column"}${serpentine ? ", turning back on each line" : ""}`;
  }
  return `LED 1 at the ${side} end, runs ${WAYS[sector(first, second)]}`;
}

export function outputName(output: SetupOutput, names: Names, session: SessionSnapshot | null): { device: string; zone: string | null } {
  const device = session ? deviceFor(session, output) : null;
  if (!device) {
    return { device: output.label, zone: output.zone_name };
  }
  const changed = session !== null && output.zone !== null && capacity(session, output) === null;
  return { device: names.device(device), zone: output.zone === null ? null : changed ? output.zone_name : names.zone(device, output.zone) };
}

export function outputLine(output: SetupOutput, names: Names, session: SessionSnapshot | null): string {
  const name = outputName(output, names, session);
  return name.zone === null ? name.device : `${name.device}, ${name.zone}`;
}

function plural(components: Component[]): string {
  const kinds = new Set(components.map((component) => kindOf(component.geometry)));
  if (kinds.size === 1 && kinds.has("Ring")) {
    return "fans";
  }
  if (kinds.size === 1 && kinds.has("Strip")) {
    return "strips";
  }
  return "components";
}

/** How a component shares its output: a mirror is one control in several places; a chain splits one zone into parts. */
export function wiringTexts(setup: Setup, id: ComponentId, names: Names, session: SessionSnapshot | null): string[] {
  const output = outputOf(setup, id);
  if (!output) {
    return ["Not wired. Choose its output in the editor."];
  }
  const texts: string[] = [];
  const shared = sharedWith(setup, id);
  if (shared.length > 0) {
    texts.push(`Mirror: one control with ${listText(shared.map((other) => nameOf(setup, other)))}. All ${countWord(shared.length + 1)} always show the same colours.`);
  }
  const chain = chainOf(setup, id);
  const part = parts(setup, output).find((each) => each.component === id);
  if (chain && part && chain.length > 1) {
    const zone = outputName(output, names, session);
    texts.push(`Part ${chain.index + 1} of ${chain.length} of a daisy chain on ${zone.zone ?? zone.device}: LEDs ${part.start + 1} to ${part.end}`);
  }
  return texts;
}

function viewName(setup: Setup, view: ViewId): string {
  return setup.views.find((each) => each.id === view)?.name ?? "";
}

export function placedLeds(setup: Setup, output: SetupOutput, view: ViewId): number {
  const covered = new Set<number>();
  for (const part of parts(setup, output)) {
    if (placementOf(setup, view, part.component)) {
      for (let led = part.start; led < part.end; led += 1) {
        covered.add(led);
      }
    }
  }
  return covered.size;
}

export function outputLines(setup: Setup, output: SetupOutput, view: ViewId, session: SessionSnapshot | null): string[] {
  const lines: string[] = [];
  const components = output.branches.flat().flatMap((id) => setup.components.find((component) => component.id === id) ?? []);
  if (output.branches.length > 1) {
    lines.push(`${components.length} ${plural(components)} mirrored: one control shown in ${output.branches.length} places`);
  }
  const chained = output.branches.filter((branch) => branch.length > 1);
  if (chained.length > 0) {
    lines.push(`${chained[0]!.length} parts in a daisy chain, one after the other on the wire`);
  }
  lines.push(`Holds ${listText(components.map((component) => component.name))}.`);
  const room = session ? capacity(session, output) : null;
  if (session && !deviceFor(session, output)) {
    lines.push("Not found today. PrismForge sends it nothing until it is back.");
    return lines;
  }
  if (session && room === null) {
    lines.push(
      `This zone is no longer ${output.zone_name ?? "the one placed in the setup"}, so the parts placed on it may not fit. Effects across the setup leave it out until you place them again.`,
    );
    return lines;
  }
  const total = room?.available ?? output.first_led + span(setup, output);
  const placed = placedLeds(setup, output, view);
  const left = total - placed;
  if (placed === 0) {
    lines.push(`Not in the ${viewName(setup, view)} view, so an effect across it leaves this output as it is.`);
  } else {
    lines.push(
      left > 0
        ? `${placed} of ${total} LEDs have a place. ${left} ${left === 1 ? "is" : "are"} not placed and stay${left === 1 ? "s" : ""} dark while an effect runs across the setup.`
        : `All ${total} LEDs have a place.`,
    );
  }
  return lines;
}

function outputTitle(setup: Setup, id: string, names: Names, session: SessionSnapshot | null): string {
  const output = setup.outputs.find((each) => each.id === id);
  if (!output) {
    return "An output";
  }
  const name = outputName(output, names, session);
  return name.zone ?? name.device;
}

/** Why the store refused the drawing, with the names the user gave. */
export function setupProblemText(problem: SetupProblem, setup: Setup, names: Names, session: SessionSnapshot | null): string {
  if ("ExceedsOutput" in problem) {
    const { output, needed, available } = problem.ExceedsOutput;
    return `The parts on ${outputTitle(setup, output, names, session)} need ${needed} LEDs, and it has ${available}.`;
  }
  if ("LedCountOutsideBounds" in problem) {
    const { component, leds: count, min, max } = problem.LedCountOutsideBounds;
    return `${nameOf(setup, component)} has ${count} LEDs. Its output takes ${min} to ${max}.`;
  }
  if ("UnknownTarget" in problem) {
    return `${outputTitle(setup, problem.UnknownTarget.output, names, session)} is not reported now. Wire it to a device or zone that is reported now.`;
  }
  if ("OverlappingParts" in problem) {
    const { first, second } = problem.OverlappingParts;
    return `${outputTitle(setup, first, names, session)} and ${outputTitle(setup, second, names, session)} would use the same LEDs. Wire the second component as a mirror or as the next part of the chain instead.`;
  }
  if ("InvalidName" in problem) {
    return `A name is not valid. ${nameProblemText(problem.InvalidName.problem)}`;
  }
  if ("InvalidGeometry" in problem) {
    return `${nameOf(setup, problem.InvalidGeometry.component)} needs 1 to ${COMPONENT_LED_LIMIT} LEDs and a shape with a size.`;
  }
  if ("InvalidPlacement" in problem) {
    return `${nameOf(setup, problem.InvalidPlacement.component)} needs a size above 0 and a turn below 360°.`;
  }
  if ("TooMany" in problem) {
    return `A setup holds at most ${problem.TooMany.limit} ${problem.TooMany.part.toLowerCase()}.`;
  }
  return "The setup is not consistent, so PrismForge did not store it. Undo the last change, then try again.";
}

/** What Identify lights for a component: always its whole device or zone, which may hold other components too. */
export function identifyText(setup: Setup, id: ComponentId, names: Names, session: SessionSnapshot | null): string {
  const output = outputOf(setup, id);
  if (!output) {
    return "Identify needs an output. Wire the component first.";
  }
  const name = outputName(output, names, session);
  const where = name.zone === null ? `all of ${name.device}` : `all of ${name.zone} on ${name.device}`;
  const together = output.branches.flat();
  if (together.length === 1) {
    return `Identify lights ${where}.`;
  }
  return `Identify lights ${where}, so ${listText(together.map((each) => nameOf(setup, each)))} light together. PrismForge cannot light one part of a zone alone.`;
}

/** Why an output got nothing from an apply across the setup; one wording for Setup and Effects. */
export function excludedText(status: SetupOutputStatus): string {
  const where = status.zone_name ? `${status.zone_name} on ${status.label}` : status.label;
  const why =
    status.problem !== null
      ? "The parts placed on it no longer fit the LEDs it has now. Change them in Setup."
      : `This zone is no longer ${status.zone_name ?? "the one placed in the setup"}, so the parts placed on it may not fit. Place them again in Setup on the zone it is on now.`;
  return `${where}: not sent. ${why}`;
}
