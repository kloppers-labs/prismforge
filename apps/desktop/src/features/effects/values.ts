import type { ControlDefinition } from "../../lib/api/bindings/ControlDefinition";
import type { ControlValue } from "../../lib/api/bindings/ControlValue";
import type { Direction } from "../../lib/api/bindings/Direction";
import type { EffectDefinition } from "../../lib/api/bindings/EffectDefinition";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import type { ValueProblem } from "../../lib/api/bindings/ValueProblem";

export type Field =
  | { kind: "color"; text: string }
  | { kind: "palette"; texts: string[] }
  | { kind: "number"; text: string }
  | { kind: "direction"; value: Direction };

export type Fields = Record<string, Field>;

export const COLOR_EXAMPLE = "#ff8000";

export function toHex(color: Rgb): string {
  return `#${[color.r, color.g, color.b].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
}

export function parseHex(text: string): Rgb | null {
  const match = /^#?([0-9a-f]{6})$/i.exec(text.trim());
  if (!match?.[1]) {
    return null;
  }
  const value = Number.parseInt(match[1], 16);
  return { r: (value >> 16) & 255, g: (value >> 8) & 255, b: value & 255 };
}

export function initialFields(definition: EffectDefinition, values: Record<string, ControlValue> = {}): Fields {
  return Object.fromEntries(definition.controls.map((control) => [control.id, field(control, values[control.id])]));
}

function field(control: ControlDefinition, value: ControlValue | undefined): Field {
  const kind = control.kind;
  if ("Color" in kind) {
    const color = value && "Color" in value ? value.Color : kind.Color.default;
    return { kind: "color", text: color ? toHex(color) : "" };
  }
  if ("Palette" in kind) {
    const colors = value && "Palette" in value ? value.Palette : kind.Palette.default;
    return { kind: "palette", texts: colors.map(toHex) };
  }
  if ("Direction" in kind) {
    return { kind: "direction", value: value && "Direction" in value ? value.Direction : kind.Direction.default };
  }
  const number = value && "Number" in value ? value.Number : numberKind(control).default;
  return { kind: "number", text: String(number) };
}

export type NumberKind = { min: number; max: number; step: number; default: number; unit: string };

export function numberKind(control: ControlDefinition): NumberKind {
  const kind = control.kind;
  if ("Speed" in kind) {
    return { ...kind.Speed.range, step: 1, default: kind.Speed.default, unit: "cycles per minute" };
  }
  if ("Brightness" in kind) {
    return { ...kind.Brightness.range, step: 1, default: kind.Brightness.default, unit: "%" };
  }
  if ("Number" in kind) {
    const number = kind.Number;
    return { ...number.range, step: Math.max(number.step, 1), default: number.default, unit: number.unit === "Percent" ? "%" : "" };
  }
  throw new Error(`control ${control.id} is not a number`);
}

function parseWhole(text: string): number | null {
  return /^\d+$/.test(text.trim()) ? Number(text.trim()) : null;
}

/** The value Rust would receive, or the client-side problem that mirrors its validation. */
export function convert(control: ControlDefinition, field: Field): { value: ControlValue } | { problem: string } {
  const kind = control.kind;
  if (field.kind === "color") {
    if (field.text.trim() === "") {
      return { problem: `Required. Enter a colour like ${COLOR_EXAMPLE}.` };
    }
    const color = parseHex(field.text);
    return color ? { value: { Color: color } } : { problem: hexProblem() };
  }
  if (field.kind === "palette" && "Palette" in kind) {
    const colors = field.texts.map(parseHex);
    if (colors.some((color) => color === null)) {
      return { problem: hexProblem() };
    }
    const { min_colors: min, max_colors: max } = kind.Palette;
    if (colors.length < min || colors.length > max) {
      return { problem: paletteProblem(min, max) };
    }
    return { value: { Palette: colors as Rgb[] } };
  }
  if (field.kind === "direction") {
    return { value: { Direction: field.value } };
  }
  if (field.kind === "number") {
    const range = numberKind(control);
    const number = parseWhole(field.text);
    if (number === null) {
      return { problem: "Enter a whole number." };
    }
    if (number < range.min || number > range.max) {
      return { problem: rangeProblem(range.min, range.max) };
    }
    if ((number - range.min) % range.step !== 0) {
      return { problem: stepProblem(range.step) };
    }
    return { value: { Number: number } };
  }
  return { problem: "This value has the wrong type." };
}

export type Checked = { instance: EffectInstance | null; problems: Record<string, string> };

export function check(definition: EffectDefinition, fields: Fields): Checked {
  const values: Record<string, ControlValue> = {};
  const problems: Record<string, string> = {};
  for (const control of definition.controls) {
    const current = fields[control.id];
    if (!current) {
      continue;
    }
    const result = convert(control, current);
    if ("value" in result) {
      values[control.id] = result.value;
    } else {
      problems[control.id] = result.problem;
    }
  }
  const valid = Object.keys(problems).length === 0;
  return { instance: valid ? { effect: definition.id, values } : null, problems };
}

export function problemText(problem: ValueProblem): string {
  if (problem === "Missing") {
    return "Required.";
  }
  if (problem === "UnknownControl") {
    return "This effect has no setting with this name.";
  }
  if ("WrongType" in problem) {
    return "This value has the wrong type.";
  }
  if ("OutOfRange" in problem) {
    return rangeProblem(problem.OutOfRange.min, problem.OutOfRange.max);
  }
  if ("OffStep" in problem) {
    return stepProblem(problem.OffStep.step);
  }
  return paletteProblem(problem.PaletteLength.min, problem.PaletteLength.max);
}

function hexProblem(): string {
  return `Enter a colour as # and six hex digits, like ${COLOR_EXAMPLE}.`;
}

function rangeProblem(min: number, max: number): string {
  return `Use a value from ${min} to ${max}.`;
}

function stepProblem(step: number): string {
  return `Use steps of ${step}.`;
}

function paletteProblem(min: number, max: number): string {
  return `Use ${min} to ${max} colours.`;
}
