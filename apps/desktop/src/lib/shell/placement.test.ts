import { expect, test } from "vitest";
import { menuPlacement, panelOffset } from "./placement";

test("a_panel_that_fits_stays_under_its_chip", () => {
  expect(panelOffset({ triggerLeft: 100, panelWidth: 300, viewportWidth: 920 })).toBe(0);
});

test("a_panel_that_would_leave_the_window_moves_left_to_fit", () => {
  expect(panelOffset({ triggerLeft: 354, panelWidth: 310, viewportWidth: 640 })).toBe(640 - 8 - 310 - 354);
});

test("a_panel_never_starts_left_of_the_window_margin", () => {
  expect(panelOffset({ triggerLeft: 80, panelWidth: 700, viewportWidth: 460 })).toBe(8 - 80);
});

const VIEW = { width: 920, height: 600 };
const SIZE = { width: 240, height: 180 };

function at(x: number, y: number) {
  return { left: x, top: y, right: x, bottom: y };
}

test("a_menu_opens_at_the_pointer_when_it_fits", () => {
  expect(menuPlacement(at(300, 200), SIZE, VIEW)).toEqual({ left: 300, top: 200 });
});

test("a_menu_near_the_right_and_bottom_edges_flips_to_the_pointer's_other_side", () => {
  expect(menuPlacement(at(900, 590), SIZE, VIEW)).toEqual({ left: 900 - 240, top: 590 - 180 });
});

test("a_keyboard_menu_opens_below_its_item_or_above_it_when_there_is_no_room", () => {
  const row = { left: 80, top: 120, right: 340, bottom: 160 };
  expect(menuPlacement(row, SIZE, VIEW)).toEqual({ left: 80, top: 160 });
  const low = { left: 80, top: 500, right: 340, bottom: 540 };
  expect(menuPlacement(low, SIZE, VIEW)).toEqual({ left: 80, top: 500 - 180 });
});

test("a_menu_taller_or_wider_than_the_room_on_both_sides_is_clamped_inside_the_margin", () => {
  const small = { width: 460, height: 300 };
  expect(menuPlacement(at(200, 150), { width: 300, height: 260 }, small)).toEqual({ left: 460 - 8 - 300, top: 300 - 8 - 260 });
  expect(menuPlacement(at(10, 10), { width: 444, height: 284 }, small)).toEqual({ left: 8, top: 8 });
});
