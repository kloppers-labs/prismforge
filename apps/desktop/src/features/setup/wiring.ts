import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { OutputId } from "../../lib/api/bindings/OutputId";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupOutput } from "../../lib/api/bindings/SetupOutput";
import { ledCount } from "./geometry";

/** The LEDs of an output that one component shows, by LED index in its device or zone. */
export type Part = { component: ComponentId; branch: number; start: number; end: number };

export type Chain = { output: OutputId; branch: number; index: number; length: number };

export function componentLeds(setup: Setup, id: ComponentId): number {
  const component = setup.components.find((each) => each.id === id);
  return component ? ledCount(component.geometry) : 0;
}

export function parts(setup: Setup, output: SetupOutput): Part[] {
  return output.branches.flatMap((components, branch) => {
    let start = output.first_led;
    return components.map((component) => {
      const end = start + componentLeds(setup, component);
      const part = { component, branch, start, end };
      start = end;
      return part;
    });
  });
}

/** LEDs the output uses from its first LED: the longest of its branches. */
export function span(setup: Setup, output: SetupOutput): number {
  return Math.max(0, ...output.branches.map((branch) => branch.reduce((sum, id) => sum + componentLeds(setup, id), 0)));
}

export function outputOf(setup: Setup, id: ComponentId): SetupOutput | null {
  return setup.outputs.find((output) => output.branches.some((branch) => branch.includes(id))) ?? null;
}

export function partOf(setup: Setup, id: ComponentId): Part | null {
  const output = outputOf(setup, id);
  return output ? (parts(setup, output).find((part) => part.component === id) ?? null) : null;
}

/** Components on other branches of the same output whose LEDs overlap: they always show the same colours. */
export function sharedWith(setup: Setup, id: ComponentId): ComponentId[] {
  const output = outputOf(setup, id);
  if (!output) {
    return [];
  }
  const all = parts(setup, output);
  const own = all.find((part) => part.component === id)!;
  return all.filter((other) => other.branch !== own.branch && other.start < own.end && own.start < other.end).map((other) => other.component);
}

export function chainOf(setup: Setup, id: ComponentId): Chain | null {
  const output = outputOf(setup, id);
  if (!output) {
    return null;
  }
  const branch = output.branches.findIndex((each) => each.includes(id));
  const components = output.branches[branch]!;
  return { output: output.id, branch, index: components.indexOf(id), length: components.length };
}

export function targetOf(output: SetupOutput): OutputTarget {
  return { device: output.device, zone: output.zone };
}

export function sameTarget(left: OutputTarget, right: OutputTarget): boolean {
  return left.device === right.device && left.zone === right.zone;
}

export function outputFor(setup: Setup, target: OutputTarget): SetupOutput | null {
  return setup.outputs.find((output) => sameTarget(targetOf(output), target)) ?? null;
}
