import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers, openEffects, wait, withoutEffectsOn, withOwner } from "../../test/effects";
import { demoEffectIdentifying, demoEffectsNotConnected, demoEffectsRunning } from "../../test/fixtures/sessions";

const DEBOUNCE_AND_MARGIN = 700;
const LIVE_HINT = /Changes apply as you make them/;

type Shell = ReturnType<typeof fakeShell>;

function applies(shell: Shell) {
  return shell.calls.filter((call) => call.command === "apply_effect");
}

function order(shell: Shell) {
  const reads = ["view", "effect_catalog", "preview_effect", "profile_store"];
  return shell.commands().filter((command) => !reads.includes(command));
}

async function setBrightness(value: string) {
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value } });
}

async function runningGraphicsCard(handlers: Parameters<typeof effectHandlers>[0] = {}) {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ apply_effect: () => null, ...handlers }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  expect(screen.getByText(LIVE_HINT)).toBeTruthy();
  return shell;
}

test("a_live_change_reapplies_once_per_debounce_window", async () => {
  const shell = await runningGraphicsCard();

  await setBrightness("40");
  await setBrightness("45");
  await wait(DEBOUNCE_AND_MARGIN);
  await setBrightness("60");
  await wait(DEBOUNCE_AND_MARGIN);

  const brightness = applies(shell).map((call) => (call.args as { effect: { values: { brightness: unknown } } }).effect.values.brightness);
  expect(brightness).toEqual([{ Number: 45 }, { Number: 60 }]);
});

test("sends_no_apply_for_a_change_made_while_stop_is_in_flight", async () => {
  const shell = await runningGraphicsCard({
    stop_effect: () => new Promise((resolve) => setTimeout(() => resolve("Restored"), 150)),
  });

  await fireEvent.click(button("Stop effect"));
  await setBrightness("40");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(order(shell)).toEqual(["stop_effect"]);
});

test("sends_no_apply_for_a_change_made_after_stop_but_before_the_next_view", async () => {
  const shell = await runningGraphicsCard({ stop_effect: () => "Restored" });

  await fireEvent.click(button("Stop effect"));
  await screen.findByText(/^Effect stopped\./);
  await setBrightness("40");
  await wait(40);
  await shell.push(withoutEffectsOn(demoEffectsRunning, "demo-graphics-card"));
  await wait(DEBOUNCE_AND_MARGIN);

  expect(order(shell)).toEqual(["stop_effect"]);
  expect(screen.queryByText(LIVE_HINT)).toBeNull();
});

test("sends_no_apply_after_a_hardware_mode_is_selected", async () => {
  const shell = await runningGraphicsCard({ select_mode: () => null });

  await setBrightness("40");
  await fireEvent.click(button("Use Breathing"));
  await setBrightness("50");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(order(shell)).toEqual(["select_mode"]);
});

test("sends_no_apply_after_another_program_is_declared", async () => {
  const shell = await runningGraphicsCard({ declare_external: () => null });

  await setBrightness("40");
  await fireEvent.click(button("Another program controls this device"));
  await setBrightness("50");
  await wait(DEBOUNCE_AND_MARGIN);
  await shell.push(withOwner(demoEffectsRunning, "demo-graphics-card", { External: { label: "another program" } }));
  await wait(100);

  expect(order(shell)).toEqual(["declare_external"]);
});

test("sends_no_apply_while_an_identify_runs_on_the_device", async () => {
  const shell = fakeShell(demoEffectIdentifying, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  await setBrightness("40");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(applies(shell)).toHaveLength(0);
  expect(screen.queryByText(LIVE_HINT)).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("sends_no_apply_while_not_connected", async () => {
  const shell = fakeShell(demoEffectsNotConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  await setBrightness("40");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(applies(shell)).toHaveLength(0);
  expect(screen.queryByText(LIVE_HINT)).toBeNull();
});

test("switching_target_cancels_a_pending_change_and_holds_it_until_apply", async () => {
  const shell = await runningGraphicsCard();

  await setBrightness("40");
  await chooseTarget("Demo Light Bar");
  await wait(DEBOUNCE_AND_MARGIN);
  expect(applies(shell)).toHaveLength(0);

  await chooseTarget("Demo Graphics Card");
  expect(screen.queryByText(LIVE_HINT)).toBeNull();
  await setBrightness("50");
  await wait(DEBOUNCE_AND_MARGIN);
  expect(applies(shell)).toHaveLength(0);

  await fireEvent.click(button("Apply effect"));
  expect(applies(shell)).toHaveLength(1);
});

test("leaving_the_effects_screen_cancels_a_pending_change", async () => {
  const shell = await runningGraphicsCard();

  await setBrightness("40");
  await fireEvent.click(screen.getByRole("button", { name: "Devices" }));
  await wait(DEBOUNCE_AND_MARGIN);
  expect(applies(shell)).toHaveLength(0);

  await openEffects();
  await setBrightness("50");
  await wait(DEBOUNCE_AND_MARGIN);
  expect(applies(shell)).toHaveLength(0);
});

test("disconnecting_cancels_a_pending_change", async () => {
  const shell = await runningGraphicsCard({ disconnect: () => shell.view(null) });

  await setBrightness("40");
  await fireEvent.click(screen.getByRole("button", { name: "Disconnect" }));
  await wait(DEBOUNCE_AND_MARGIN);

  expect(applies(shell)).toHaveLength(0);
});
