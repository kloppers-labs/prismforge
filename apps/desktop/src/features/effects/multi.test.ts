import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell, type Call } from "../../test/backend";
import { button, effectHandlers, openEffects, withEffect } from "../../test/effects";
import { batchAppliedWithSkips, batchDuringIdentify, batchStopped, stopAllReport } from "../../test/fixtures/batches";
import { demoBatchRunning, demoConnected, demoIdentifyingGpu } from "../../test/fixtures/sessions";
import { viewApplying } from "../../test/fixtures/views";

function option(device: string, part = "Whole device", index = 0): HTMLElement {
  const targets = screen.getByRole("listbox", { name: "Targets" });
  const group = within(targets).getAllByRole("group", { name: device })[index]!;
  return within(group).getByRole("option", { name: new RegExp(`^${part}`) });
}

function args(calls: Call[], command: string) {
  return calls.filter((call) => call.command === command).map((call) => call.args);
}

function sent(calls: Call[]): string[] {
  return calls.map((call) => call.command).filter((command) => !["view", "groups", "effect_catalog", "preview_effect", "profile_store", "lifecycle"].includes(command));
}

test("arrow_keys_move_focus_without_selecting_and_space_toggles_the_focused_target", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();
  const first = option("Demo Memory Module");
  first.focus();

  await fireEvent.keyDown(first, { key: "ArrowDown" });
  await fireEvent.keyDown(document.activeElement!, { key: "ArrowDown" });

  expect(document.activeElement).toBe(option("Demo Motherboard"));
  expect(screen.getAllByRole("option").filter((row) => row.getAttribute("aria-selected") === "true")).toHaveLength(0);
  expect(screen.getByText("No target selected")).toBeTruthy();
  expect(sent(shell.calls)).toEqual([]);

  await fireEvent.keyDown(document.activeElement!, { key: " " });

  expect(option("Demo Motherboard").getAttribute("aria-selected")).toBe("true");
  expect(screen.getByText("1 target selected")).toBeTruthy();
  expect(sent(shell.calls)).toEqual([]);
});

test("shift_arrow_extends_the_selection_and_control_a_selects_every_target_that_can_take_an_effect", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();
  const first = option("Demo Memory Module");
  first.focus();

  await fireEvent.keyDown(first, { key: " " });
  await fireEvent.keyDown(first, { key: "ArrowDown", shiftKey: true });

  expect(screen.getByText("2 targets selected")).toBeTruthy();
  expect(await screen.findByRole("heading", { level: 2, name: "2 targets" })).toBeTruthy();

  await fireEvent.keyDown(document.activeElement!, { key: "a", ctrlKey: true });

  expect(screen.getByText("4 targets selected")).toBeTruthy();
  expect(option("Demo Light Bar").getAttribute("aria-selected")).toBe("false");
  expect(button("Apply to 4 targets")).toBeTruthy();
});

test("a_device_with_some_zones_selected_shows_a_mixed_state", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();

  await fireEvent.click(option("Demo Motherboard"), { ctrlKey: true });
  await fireEvent.click(option("Demo Motherboard", "Onboard"), { ctrlKey: true });

  const whole = option("Demo Motherboard");
  expect(whole.getAttribute("aria-selected")).toBe("false");
  expect(whole.getAttribute("aria-describedby")).toMatch(/^state-/);
  expect(document.getElementById(whole.getAttribute("aria-describedby")!)?.textContent).toBe("Some zones selected");
  expect(option("Demo Motherboard", "Addressable Header 1").getAttribute("aria-selected")).toBe("true");
  expect(option("Demo Motherboard", "Onboard").getAttribute("aria-selected")).toBe("false");
  expect(screen.getByText("2 targets selected")).toBeTruthy();

  await fireEvent.click(whole, { ctrlKey: true });

  expect(option("Demo Motherboard").getAttribute("aria-selected")).toBe("true");
  expect(screen.getByText("1 target selected")).toBeTruthy();
});

test("a_target_that_cannot_take_an_effect_says_why_and_stays_out_of_the_selection", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();
  const bar = option("Demo Light Bar");

  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  bar.focus();
  await fireEvent.keyDown(bar, { key: " " });
  await fireEvent.click(bar, { ctrlKey: true });

  expect(bar.getAttribute("aria-disabled")).toBe("true");
  expect(bar.getAttribute("aria-selected")).toBe("false");
  expect(document.getElementById(bar.getAttribute("aria-describedby")!)?.textContent).toMatch(/no Direct Mode/);
  expect(screen.getByText("1 target selected")).toBeTruthy();
});

test("apply_sends_one_batch_and_lists_each_target", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect_batch: () => batchAppliedWithSkips }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module"), { ctrlKey: true });
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await fireEvent.click(button("Apply to 2 targets"));

  const results = await screen.findByRole("status", { name: "Results for each target" });
  expect(args(shell.calls, "apply_effect_batch")).toHaveLength(1);
  const first = args(shell.calls, "apply_effect_batch")[0]!;
  expect(first.targets).toEqual([
    { device: "demo-graphics-card", zone: null },
    { device: "demo-memory-1", zone: null },
  ]);
  expect(within(results).getByText(/^Rainbow wave: 3 of 5 targets accepted\.$/)).toBeTruthy();
  expect(within(results).getAllByRole("listitem")).toHaveLength(6);
  expect(within(results).getByText(/Demo Light Bar: skipped\./)).toBeTruthy();
  expect(within(results).getByText(/Addressable Header 1 on Demo Motherboard: sent once, because the whole device is also selected\./)).toBeTruthy();
  expect(results.querySelector(".report")?.classList.contains("unsettled")).toBe(true);
});

test("stop_on_the_selection_and_stop_all_report_in_the_same_form", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const shell = fakeShell(demoBatchRunning, effectHandlers({ stop_effect_batch: () => batchStopped, stop_all_effects: () => stopAllReport }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module", "Whole device", 1), { ctrlKey: true });

  await fireEvent.click(button("Stop on 2 targets"));

  const results = await screen.findByRole("status", { name: "Results for each target" });
  expect(args(shell.calls, "stop_effect_batch")).toEqual([
    {
      targets: [
        { device: "demo-graphics-card", zone: null },
        { device: "demo-memory-2", zone: null },
      ],
    },
  ]);
  expect(within(results).getByText(/^1 of 2 targets stopped\.$/)).toBeTruthy();

  await fireEvent.click(button("Stop all PrismForge effects"));

  await waitFor(() => expect(within(results).getByText(/^Stop all: \d of \d devices stopped\.$/)).toBeTruthy());
  expect(sent(shell.calls)).toContain("stop_all_effects");
});

test("live_edits_with_several_targets_go_to_the_whole_selection_and_never_scroll", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const scrolled = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});
  const shell = fakeShell(demoBatchRunning, effectHandlers({ apply_effect_batch: (call) => ({ results: (call.targets as object[]).map((target) => ({ target, outcome: "Applied" })) }) }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module", "Whole device", 1), { ctrlKey: true });
  scrolled.mockClear();

  expect(await screen.findByText(/^Changes go to all 2 targets as you make them\.$/)).toBeTruthy();
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "40" } });
  await new Promise((resolve) => setTimeout(resolve, 600));

  expect(args(shell.calls, "apply_effect_batch")).toEqual([
    {
      targets: [
        { device: "demo-graphics-card", zone: null },
        { device: "demo-memory-2", zone: null },
      ],
      effect: expect.objectContaining({ effect: "rainbow-wave" }),
    },
  ]);
  expect(sent(shell.calls).filter((command) => command === "apply_effect")).toEqual([]);
  expect(scrolled).not.toHaveBeenCalled();
});

test("several_targets_cannot_be_applied_while_a_profile_applies", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Motherboard"), { ctrlKey: true });
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await shell.pushView({ ...viewApplying, session: demoConnected, revision: 99 });

  await waitFor(() => expect(button("Apply to 2 targets").disabled).toBe(true));
  expect(screen.getByText(/^Effects are off\. PrismForge is applying the profile/)).toBeTruthy();
});

test("live_edits_stay_off_while_the_targets_hold_different_values_and_apply_says_all_get_the_shown_values", async () => {
  const fast = { effect: "rainbow-wave", values: { speed: { Number: 60 } } };
  const slow = { effect: "rainbow-wave", values: {} };
  const running = withEffect(withEffect(demoConnected, { device: "demo-graphics-card", zone: null }, slow, "Running"), { device: "demo-memory-2", zone: null }, fast, "Running");
  const shell = fakeShell(running, effectHandlers({ apply_effect_batch: (call) => ({ results: (call.targets as object[]).map((target) => ({ target, outcome: "Applied" })) }) }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module", "Whole device", 1), { ctrlKey: true });

  const speed = screen.getByRole("spinbutton", { name: "Speed" });
  expect(document.getElementById(speed.getAttribute("aria-describedby")?.split(" ").find((id) => id.startsWith("mixed-")) ?? "")?.textContent).toMatch(/^Mixed\./);
  expect(screen.getByText(/^Some settings differ between the targets, marked Mixed\. Apply sends the values shown to all 2 targets\./)).toBeTruthy();
  expect(screen.queryByText(/Changes go to all 2 targets/)).toBeNull();

  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "40" } });
  await new Promise((resolve) => setTimeout(resolve, 600));
  expect(args(shell.calls, "apply_effect_batch")).toEqual([]);

  await fireEvent.click(button("Apply to 2 targets"));
  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")).toHaveLength(1));
  const effect = args(shell.calls, "apply_effect_batch")[0]!.effect as { values: Record<string, unknown> };
  expect(effect.values.speed).toEqual({ Number: 20 });
  expect(effect.values.brightness).toEqual({ Number: 40 });
});

test("every_zone_checked_stays_zone_by_zone_until_the_device_row_itself_is_checked", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect_batch: () => ({ results: [] }) }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  for (const zone of ["Onboard", "Addressable Header 1", "Addressable Header 2"]) {
    await fireEvent.click(option("Demo Motherboard", zone), { ctrlKey: true });
  }

  const whole = option("Demo Motherboard");
  expect(whole.getAttribute("aria-selected")).toBe("false");
  expect(document.getElementById(whole.getAttribute("aria-describedby")!)?.textContent).toBe("Every zone selected on its own");
  expect(within(whole).getByText("Each zone")).toBeTruthy();
  expect(screen.getByText("4 targets selected")).toBeTruthy();
  expect(screen.getByText("Demo Graphics Card (whole device), 3 zones of Demo Motherboard")).toBeTruthy();

  await fireEvent.click(whole, { ctrlKey: true });

  const one = option("Demo Motherboard");
  expect(one.getAttribute("aria-selected")).toBe("true");
  expect(document.getElementById(one.getAttribute("aria-describedby")!)?.textContent).toBe("Selected as one target");
  expect(within(one).getByText("As one")).toBeTruthy();
  expect(within(one).queryByText("Each zone")).toBeNull();
  expect(screen.getByText("Demo Graphics Card (whole device), Demo Motherboard (whole device)")).toBeTruthy();
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  await fireEvent.click(button("Apply to 2 targets"));
  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")[0]?.targets).toEqual([
    { device: "demo-graphics-card", zone: null },
    { device: "demo-motherboard", zone: null },
  ]));
});

test("a_blocked_zone_states_its_own_reason", async () => {
  const empty = structuredClone(demoConnected);
  const board = empty.devices.find((device) => device.descriptor.name === "Demo Motherboard")!;
  board.descriptor.zones[1]!.led_count = 0;
  fakeShell(empty, effectHandlers());
  render(App);
  await openEffects();

  const header = option("Demo Motherboard", "Addressable Header 1");

  expect(header.getAttribute("aria-disabled")).toBe("true");
  const reason = document.getElementById(header.getAttribute("aria-describedby")!);
  expect(reason?.textContent).toBe("This zone has no LEDs.");
  expect(reason?.classList.contains("visually-hidden")).toBe(false);
});

test("retry_resends_only_the_targets_that_can_work_later_and_counts_them", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const shell = fakeShell(demoIdentifyingGpu, effectHandlers({ apply_effect_batch: () => batchDuringIdentify }));
  render(App);
  await openEffects();
  await fireEvent.click(screen.getByRole("button", { name: "Select all" }));
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await fireEvent.click(button("Apply to 4 targets"));
  const results = await screen.findByRole("status", { name: "Results for each target" });
  await fireEvent.click(within(results).getByRole("button", { name: "Retry 1 target" }));

  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")).toHaveLength(2));
  expect(args(shell.calls, "apply_effect_batch")[1]?.targets).toEqual([{ device: "demo-graphics-card", zone: null }]);
});

test("retry_is_not_offered_for_targets_that_would_be_refused_again", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  fakeShell(demoConnected, effectHandlers({ apply_effect_batch: () => batchAppliedWithSkips }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module"), { ctrlKey: true });
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await fireEvent.click(button("Apply to 2 targets"));

  const results = await screen.findByRole("status", { name: "Results for each target" });
  expect(within(results).getByText(/Demo Light Bar: skipped\./)).toBeTruthy();
  expect(within(results).queryByRole("button", { name: /^Retry/ })).toBeNull();
});

function differing() {
  const plain = { effect: "rainbow-wave", values: {} };
  const other = { effect: "rainbow-wave", values: { speed: { Number: 60 }, brightness: { Number: 50 } } };
  return withEffect(withEffect(demoConnected, { device: "demo-graphics-card", zone: null }, plain, "Running"), { device: "demo-memory-2", zone: null }, other, "Running");
}

function batchAnswer() {
  return (call: Record<string, unknown>) => ({ results: (call.targets as object[]).map((target) => ({ target, outcome: "Applied" })) });
}

test("editing_every_mixed_setting_still_sends_nothing_until_apply", async () => {
  const shell = fakeShell(differing(), effectHandlers({ apply_effect_batch: batchAnswer() }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module", "Whole device", 1), { ctrlKey: true });
  expect(screen.getAllByText(/^Mixed\./)).toHaveLength(2);

  await fireEvent.input(screen.getByRole("spinbutton", { name: "Speed" }), { target: { value: "30" } });
  await new Promise((resolve) => setTimeout(resolve, 600));
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "70" } });
  await new Promise((resolve) => setTimeout(resolve, 600));

  expect(screen.queryAllByText(/^Mixed\./)).toHaveLength(0);
  expect(args(shell.calls, "apply_effect_batch")).toEqual([]);
  expect(screen.getByText(/^Some settings differ between the targets/)).toBeTruthy();
  expect(screen.queryByText(/Changes go to all 2 targets/)).toBeNull();

  await fireEvent.click(button("Apply to 2 targets"));
  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")).toHaveLength(1));
  const effect = args(shell.calls, "apply_effect_batch")[0]!.effect as { values: Record<string, unknown> };
  expect(effect.values.speed).toEqual({ Number: 30 });
  expect(effect.values.brightness).toEqual({ Number: 70 });
});

test("values_that_start_to_differ_under_the_same_selection_turn_live_off", async () => {
  const shell = fakeShell(demoBatchRunning, effectHandlers({ apply_effect_batch: batchAnswer() }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Memory Module", "Whole device", 1), { ctrlKey: true });
  expect(await screen.findByText(/^Changes go to all 2 targets as you make them\.$/)).toBeTruthy();

  const changed = structuredClone(demoBatchRunning);
  const memory = changed.output.scene.assignments.find((assignment) => assignment.target.device === "demo-memory-2")!;
  memory.effect.values = { ...memory.effect.values, speed: { Number: 60 } };
  await shell.push(changed);

  expect(await screen.findByText(/^Some settings differ between the targets, marked Mixed\./)).toBeTruthy();
  const speed = screen.getByRole("spinbutton", { name: "Speed" });
  expect(speed.getAttribute("aria-describedby")).toMatch(/mixed-speed/);
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "40" } });
  await new Promise((resolve) => setTimeout(resolve, 600));
  expect(args(shell.calls, "apply_effect_batch")).toEqual([]);
});
