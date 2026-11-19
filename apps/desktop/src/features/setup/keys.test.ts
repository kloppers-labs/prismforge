import { expect, test } from "vitest";
import { editAction } from "./keys";

function key(init: KeyboardEventInit) {
  return editAction(new KeyboardEvent("keydown", init));
}

test("alt_with_an_arrow_moves_by_ten_and_with_shift_by_fifty", () => {
  expect(key({ key: "ArrowLeft", altKey: true })).toEqual({ kind: "move", dx: -10, dy: 0 });
  expect(key({ key: "ArrowDown", altKey: true, shiftKey: true })).toEqual({ kind: "move", dx: 0, dy: 50 });
  expect(key({ key: "ArrowDown" })).toBeNull();
});

test("brackets_turn_shift_h_flips_and_delete_takes_out_of_the_view", () => {
  expect(key({ key: "]" })).toEqual({ kind: "turn", steps: 1 });
  expect(key({ key: "[" })).toEqual({ kind: "turn", steps: -1 });
  expect(key({ key: "H", shiftKey: true })).toEqual({ kind: "flip" });
  expect(key({ key: "h" })).toBeNull();
  expect(key({ key: "Delete" })).toEqual({ kind: "remove" });
});

test("ctrl_z_undoes_ctrl_shift_z_and_ctrl_y_redo_and_ctrl_s_saves", () => {
  expect(key({ key: "z", ctrlKey: true })).toEqual({ kind: "undo" });
  expect(key({ key: "Z", ctrlKey: true, shiftKey: true })).toEqual({ kind: "redo" });
  expect(key({ key: "y", ctrlKey: true })).toEqual({ kind: "redo" });
  expect(key({ key: "s", ctrlKey: true })).toEqual({ kind: "save" });
  expect(key({ key: "]", ctrlKey: true })).toBeNull();
});
