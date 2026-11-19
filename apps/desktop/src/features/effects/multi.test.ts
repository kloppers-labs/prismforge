import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell, type Call } from "../../test/backend";
import { button, effectHandlers, openEffects } from "../../test/effects";
import { batchAppliedWithSkips, batchStopped, stopAllReport } from "../../test/fixtures/batches";
import { demoBatchRunning, demoConnected } from "../../test/fixtures/sessions";
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
  return calls.map((call) => call.command).filter((command) => !["view", "effect_catalog", "preview_effect", "profile_store", "lifecycle"].includes(command));
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
  expect(whole.getAttribute("aria-describedby")).toMatch(/^mixed-/);
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

test("apply_sends_one_batch_and_lists_each_target_with_retry_for_the_failed_ones", async () => {
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
  expect(within(results).getByText(/Demo Motherboard, Addressable Header 1: sent once, because the whole device is also selected\./)).toBeTruthy();
  expect(results.querySelector(".report")?.classList.contains("unsettled")).toBe(true);

  await fireEvent.click(within(results).getByRole("button", { name: "Retry failed (2)" }));

  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")).toHaveLength(2));
  expect(args(shell.calls, "apply_effect_batch")[1]).toEqual({
    targets: [
      { device: "demo-memory-1", zone: null },
      { device: "demo-light-bar", zone: null },
    ],
    effect: first.effect,
  });
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
