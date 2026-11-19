import { expect, test } from "vitest";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import { demoConnected, demoEffectsRunning } from "../../test/fixtures/sessions";
import { withOwner } from "../../test/effects";
import { addRange, choosable, effective, isMixed, isSelected, options, selectAll, toggle } from "./selection";

const BOARD = "demo-motherboard";
const GPU = "demo-graphics-card";
const BAR = "demo-light-bar";

function whole(device: string): OutputTarget {
  return { device, zone: null };
}

function zone(device: string, index: number): OutputTarget {
  return { device, zone: index };
}

test("checking_a_whole_device_selects_it_and_its_zones_show_as_selected", () => {
  const selection = toggle([zone(BOARD, 1)], whole(BOARD), demoConnected);

  expect(selection).toEqual([whole(BOARD)]);
  expect(isSelected(selection, zone(BOARD, 2))).toBe(true);
  expect(isMixed(selection, BOARD)).toBe(false);
});

test("unchecking_one_zone_of_a_selected_device_leaves_the_other_zones_and_a_mixed_device", () => {
  const selection = toggle([whole(BOARD), whole(GPU)], zone(BOARD, 1), demoConnected);

  expect(selection).toEqual([whole(GPU), zone(BOARD, 0), zone(BOARD, 2)]);
  expect(isMixed(selection, BOARD)).toBe(true);
  expect(isSelected(selection, whole(BOARD))).toBe(false);
});

test("checking_the_last_zone_of_a_device_selects_the_whole_device", () => {
  const selection = toggle([zone(BOARD, 0), zone(BOARD, 2)], zone(BOARD, 1), demoConnected);

  expect(selection).toEqual([whole(BOARD)]);
});

test("a_mixed_device_becomes_whole_when_checked_and_empty_when_checked_again", () => {
  const mixed = [zone(BOARD, 0)];

  const all = toggle(mixed, whole(BOARD), demoConnected);
  const none = toggle(all, whole(BOARD), demoConnected);

  expect(all).toEqual([whole(BOARD)]);
  expect(none).toEqual([]);
});

test("a_target_that_cannot_take_an_effect_cannot_be_checked_and_states_why", () => {
  const external = withOwner(demoConnected, GPU, { External: { label: "OpenRGB Effects plugin" } });
  const listed = options(external);

  expect(listed.find((option) => option.target.device === BAR)?.reason).toMatch(/no Direct Mode/);
  expect(listed.find((option) => option.target.device === GPU)?.reason).toMatch(/OpenRGB Effects plugin controls this device/);
  expect(toggle([], whole(BAR), external)).toEqual([]);
  expect(toggle([], whole(GPU), external)).toEqual([]);
  expect(choosable(external, whole(BAR))).toBe(false);
});

test("select_all_takes_every_target_that_can_take_an_effect_as_whole_devices", () => {
  const selection = selectAll(demoConnected);

  expect(selection.map((target) => target.device).sort()).toEqual(
    ["demo-graphics-card", "demo-memory-1", "demo-memory-2", "demo-motherboard"].sort(),
  );
  expect(selection.every((target) => target.zone === null)).toBe(true);
});

test("a_range_adds_every_choosable_target_between_two_rows", () => {
  const order = options(demoConnected).map((option) => option.target);
  const from = order.findIndex((target) => target.device === BOARD && target.zone === 0);
  const to = order.findIndex((target) => target.device === GPU);

  const selection = addRange([], order[from] ?? whole(BOARD), order[to] ?? whole(GPU), demoConnected);

  expect(selection).toContainEqual(whole(BOARD));
  expect(selection).toContainEqual(whole(GPU));
  expect(selection.some((target) => target.device === BAR)).toBe(false);
});

test("the_selection_in_use_drops_blocked_targets_but_keeps_a_single_one_to_show_it", () => {
  const external = withOwner(demoEffectsRunning, GPU, { External: { label: "Other" } });

  expect(effective([whole(GPU), whole("demo-memory-1")], external)).toEqual([whole("demo-memory-1")]);
  expect(effective([whole(GPU)], external)).toEqual([whole(GPU)]);
  expect(effective([whole("gone-device"), whole("demo-memory-1")], external)).toEqual([whole("demo-memory-1")]);
});
