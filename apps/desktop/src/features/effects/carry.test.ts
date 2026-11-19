import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers, openEffects, wait } from "../../test/effects";
import { acrossDemoSkipped, acrossDemoTower } from "../../test/fixtures/acrossReports";
import { demoAcrossBesideEachDevice, demoAcrossRunning } from "../../test/fixtures/sessions";
import { setupDemoTower } from "../../test/fixtures/setups";

const DEBOUNCE_AND_MARGIN = 700;
const ACROSS = { view: "v-pc", flow: { Along: { angle: 0 } } };

function handlers() {
  return effectHandlers({
    setup: () => setupDemoTower,
    apply_across: () => acrossDemoTower,
    apply_effect: () => null,
    apply_effect_batch: () => ({ results: [] }),
  });
}

function sent(shell: ReturnType<typeof fakeShell>) {
  return shell.commands().filter((command) => command.startsWith("apply_"));
}

async function setBrightness(value: string) {
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value } });
}

test("a_live_edit_on_a_target_running_across_the_setup_keeps_its_view_and_flow", async () => {
  const shell = fakeShell(demoAcrossRunning, handlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  await setBrightness("40");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(sent(shell)).toEqual(["apply_across"]);
  const args = shell.calls.find((call) => call.command === "apply_across")!.args;
  expect({ view: args.view, flow: args.flow }).toEqual(ACROSS);
  expect(args.targets).toEqual([{ device: "demo-graphics-card", zone: null }]);
  expect((args.effect as { values: { brightness: unknown } }).values.brightness).toEqual({ Number: 40 });
});

test("apply_on_several_targets_running_across_the_setup_keeps_the_mode", async () => {
  const shell = fakeShell(demoAcrossRunning, handlers());
  render(App);
  const targets = await openEffects();
  const row = (device: string, index = 0) => within(within(targets).getAllByRole("group", { name: device })[index]!).getByRole("option", { name: /^Whole device/ });
  await fireEvent.click(row("Demo Graphics Card"));
  await fireEvent.click(row("Demo Memory Module"), { ctrlKey: true });

  await fireEvent.click(button("Apply to 2 targets"));
  await wait(50);

  expect(sent(shell)).toEqual(["apply_across"]);
  const args = shell.calls.find((call) => call.command === "apply_across")!.args;
  expect({ view: args.view, flow: args.flow }).toEqual(ACROSS);
  expect(args.targets).toEqual([
    { device: "demo-graphics-card", zone: null },
    { device: "demo-memory-1", zone: null },
  ]);
});

test("the_effect_bar_names_the_mode_and_its_flow", async () => {
  fakeShell(demoAcrossRunning, handlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  const bar = screen.getByRole("region", { name: "Effect actions" });

  expect(bar.textContent).toContain("Runs across the setup: PC view, flowing left to right (0°)");
});

test("targets_running_in_different_modes_are_not_sent_in_either_mode", async () => {
  const shell = fakeShell(demoAcrossBesideEachDevice, handlers());
  render(App);
  const targets = await openEffects();
  const row = (device: string, index = 0) => within(within(targets).getAllByRole("group", { name: device })[index]!).getByRole("option", { name: /^Whole device/ });
  await fireEvent.click(row("Demo Graphics Card"));
  await fireEvent.click(row("Demo Memory Module", 1), { ctrlKey: true });

  const apply = button("Apply to 2 targets");
  await setBrightness("40");
  await wait(DEBOUNCE_AND_MARGIN);

  expect(apply.disabled).toBe(true);
  expect(screen.getByRole("region", { name: "Effect actions" }).textContent).toContain(
    "These targets run in different modes: some across the setup, some on each device. Select targets of one mode, or stop them first.",
  );
  expect(sent(shell)).toEqual([]);
});

test("an_apply_across_from_effects_lists_the_targets_that_got_nothing_and_retries_in_the_same_mode", async () => {
  const shell = fakeShell(demoAcrossRunning, { ...handlers(), apply_across: () => acrossDemoSkipped });
  render(App);
  const targets = await openEffects();
  const row = (device: string, index = 0) => within(within(targets).getAllByRole("group", { name: device })[index]!).getByRole("option", { name: /^Whole device/ });
  await fireEvent.click(row("Demo Graphics Card"));
  await fireEvent.click(row("Demo Memory Module"), { ctrlKey: true });
  await fireEvent.click(button("Apply to 2 targets"));

  const results = await screen.findByRole("status", { name: "Results for each target" });
  expect(results.textContent).toContain("Board logo on Demo Motherboard: not sent. This zone is no longer Board logo");
  await fireEvent.click(within(results).getByRole("button", { name: "Retry 1 target" }));
  await wait(50);

  const retry = shell.calls.filter((call) => call.command === "apply_across")[1]!.args;
  expect({ view: retry.view, flow: retry.flow, targets: retry.targets }).toEqual({ ...ACROSS, targets: [{ device: "demo-graphics-card", zone: null }] });
});
