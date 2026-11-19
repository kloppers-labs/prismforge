import { expect, test } from "vitest";
import { Names } from "../../lib/names";
import { skipText } from "./present";

test("a_setting_skipped_because_the_setup_changed_says_so", () => {
  const labels = { names: new Names(), session: null, effect: () => null };

  expect(skipText(labels, "SetupChanged")).toBe(
    "The setup changed. The setup view this effect runs across is gone, or this device is no longer placed in it.",
  );
});

test("a_setting_skipped_because_the_setup_needs_a_newer_prismforge_says_so", () => {
  const labels = { names: new Names(), session: null, effect: () => null };

  expect(skipText(labels, "NewerSetup")).toBe(
    "The setup file needs a newer PrismForge, so this effect across the setup cannot run.",
  );
});
