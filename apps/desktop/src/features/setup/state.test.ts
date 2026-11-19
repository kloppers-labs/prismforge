import { expect, test } from "vitest";
import type { SetupStoreView } from "../../lib/api/bindings/SetupStoreView";
import { fakeShell, refuse } from "../../test/backend";
import { errorSetupLedBounds } from "../../test/fixtures/errors";
import { demoConnected, openRgbSetupDevices } from "../../test/fixtures/sessions";
import { setupDemoEmpty, setupDemoTower, setupRealTower } from "../../test/fixtures/setups";
import { VIEWS, rotateBy, setGeometry } from "./edits";
import { SetupState } from "./state.svelte";

const tower = () => structuredClone(setupDemoTower);

async function loaded(view: SetupStoreView, session = demoConnected) {
  const shell = fakeShell(session, { setup: () => view, save_setup: (args) => ({ ...view, setup: args.setup }) });
  const state = new SetupState(() => session);
  await state.load();
  return { state, shell };
}

test("an_edit_marks_unsaved_changes_and_save_stores_the_drawing", async () => {
  const { state, shell } = await loaded(tower());

  state.edit("Rotate Graphics card", (setup) => rotateBy(setup, VIEWS.pc, ["c-graphics"], 1));
  const unsaved = state.dirty;
  await state.save();

  expect(unsaved).toBe(true);
  expect(state.dirty).toBe(false);
  expect(state.justSaved).toBe(true);
  const saved = shell.calls.find((call) => call.command === "save_setup")!.args.setup as SetupStoreView["setup"];
  expect(saved.views[0]!.placements.find((placement) => placement.component === "c-graphics")!.rotation).toBe(15);
});

test("undo_restores_the_drawing_and_the_selection_and_names_the_step", async () => {
  const { state } = await loaded(tower());
  state.select(["c-graphics"]);
  state.edit("Rotate Graphics card", (setup) => rotateBy(setup, VIEWS.pc, ["c-graphics"], 1), ["c-graphics", "c-rear-fan"]);

  const label = state.undoLabel;
  state.undo();

  expect(label).toBe("Rotate Graphics card");
  expect(state.dirty).toBe(false);
  expect(state.selection).toEqual(["c-graphics"]);
  expect(state.redoLabel).toBe("Rotate Graphics card");
  state.redo();
  expect(state.dirty).toBe(true);
  expect(state.selection).toEqual(["c-graphics", "c-rear-fan"]);
});

test("an_edit_that_changes_nothing_adds_no_undo_step", async () => {
  const { state } = await loaded(tower());

  state.edit("Rotate nothing", (setup) => setup);

  expect(state.undoLabel).toBeNull();
});

test("discard_returns_to_the_stored_setup_and_clears_the_history", async () => {
  const { state } = await loaded(tower());
  state.edit("Rotate Graphics card", (setup) => rotateBy(setup, VIEWS.pc, ["c-graphics"], 1));

  state.discard();

  expect(state.dirty).toBe(false);
  expect(state.undoLabel).toBeNull();
});

test("a_save_the_store_refuses_keeps_the_drawing_and_says_why", async () => {
  const shell = fakeShell(demoConnected, { setup: () => tower(), save_setup: refuse(errorSetupLedBounds) });
  const state = new SetupState(() => demoConnected);
  await state.load();
  state.edit("Change LED count", (setup) => setGeometry(setup, "c-cooler-left", { Strip: { leds: 9, first: "AtStart" } }));

  await state.save();

  expect(shell.commands()).toContain("save_setup");
  expect(state.dirty).toBe(true);
  expect(state.saveProblem).toEqual(errorSetupLedBounds);
});

test("a_new_setup_starts_with_a_pc_view_and_a_desk_view_and_nothing_unsaved", async () => {
  const { state } = await loaded(setupDemoEmpty);

  expect(state.draft.views.map((view) => view.name)).toEqual(["PC", "Desk"]);
  expect(state.dirty).toBe(false);
});

test("an_unsaved_drawing_waits_while_the_other_store_is_in_use", async () => {
  let view: SetupStoreView = tower();
  fakeShell(demoConnected, { setup: () => view });
  const state = new SetupState(() => demoConnected);
  await state.load();
  state.edit("Rotate Graphics card", (setup) => rotateBy(setup, VIEWS.pc, ["c-graphics"], 1));

  view = structuredClone(setupRealTower);
  await state.load();
  const real = { dirty: state.dirty, components: state.draft.components.length };
  view = tower();
  await state.load();

  expect(real).toEqual({ dirty: false, components: setupRealTower.setup.components.length });
  expect(state.dirty).toBe(true);
  expect(state.undoLabel).toBe("Rotate Graphics card");
});

test("the_problems_follow_the_devices_the_session_reports", async () => {
  const { state } = await loaded(tower(), openRgbSetupDevices);

  state.edit("Change LED count", (setup) => setGeometry(setup, "c-cooler-left", { Strip: { leds: 9, first: "AtStart" } }));

  expect(state.issues).toEqual([]);
});
