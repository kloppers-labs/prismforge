import { isMac } from "../../lib/platform";
import { NUDGE } from "./edits";

export const BIG_NUDGE = 50;

export type EditAction =
  | { kind: "move"; dx: number; dy: number }
  | { kind: "turn"; steps: number }
  | { kind: "flip" }
  | { kind: "remove" }
  | { kind: "undo" }
  | { kind: "redo" }
  | { kind: "save" };

const ARROWS: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };

function command(event: KeyboardEvent): boolean {
  return isMac() ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
}

/** Keys that edit the drawing from the list; the plain arrows move focus there, so moving takes Alt. */
export function editAction(event: KeyboardEvent): EditAction | null {
  if (command(event) && !event.altKey) {
    const key = event.key.toLowerCase();
    if (key === "z") {
      return { kind: event.shiftKey ? "redo" : "undo" };
    }
    if (key === "y" && !event.shiftKey) {
      return { kind: "redo" };
    }
    if (key === "s" && !event.shiftKey) {
      return { kind: "save" };
    }
    return null;
  }
  if (event.ctrlKey || event.metaKey) {
    return null;
  }
  const arrow = ARROWS[event.key];
  if (arrow && event.altKey) {
    const step = event.shiftKey ? BIG_NUDGE : NUDGE;
    return { kind: "move", dx: arrow[0] * step, dy: arrow[1] * step };
  }
  if (event.altKey) {
    return null;
  }
  if (event.key === "[" || event.key === "]") {
    return { kind: "turn", steps: event.key === "]" ? 1 : -1 };
  }
  if (event.key.toLowerCase() === "h" && event.shiftKey) {
    return { kind: "flip" };
  }
  if (event.key === "Delete" || event.key === "Backspace") {
    return { kind: "remove" };
  }
  return null;
}

export function keyHelp(): string {
  const command = isMac() ? "Command" : "Ctrl";
  return `Alt with an arrow key moves the selection by ${NUDGE}, with Shift by ${BIG_NUDGE}. [ and ] turn it by 15°, Shift+H flips it, Delete takes it out of this view. ${command}+Z undoes, ${command}+Shift+Z redoes.`;
}
