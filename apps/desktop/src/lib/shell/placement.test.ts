import { expect, test } from "vitest";
import { panelOffset } from "./placement";

test("a_panel_that_fits_stays_under_its_chip", () => {
  expect(panelOffset({ triggerLeft: 100, panelWidth: 300, viewportWidth: 920 })).toBe(0);
});

test("a_panel_that_would_leave_the_window_moves_left_to_fit", () => {
  expect(panelOffset({ triggerLeft: 354, panelWidth: 310, viewportWidth: 640 })).toBe(640 - 8 - 310 - 354);
});

test("a_panel_never_starts_left_of_the_window_margin", () => {
  expect(panelOffset({ triggerLeft: 80, panelWidth: 700, viewportWidth: 460 })).toBe(8 - 80);
});
