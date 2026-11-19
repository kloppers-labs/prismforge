import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers, openEffects } from "../../test/effects";
import { demoConnected, demoEffectsRunning } from "../../test/fixtures/sessions";
import { stopRestored } from "../../test/fixtures/stops";
import { itemLabels, openMenu, pick } from "../../test/menus";

function args(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command).map((call) => call.args);
}

async function targetRow(device: string, part = "Whole device", index = 0) {
  const targets = await openEffects();
  const group = within(targets).getAllByRole("group", { name: device })[index]!;
  return within(group).getByRole("button", { name: new RegExp(`^${part}`) }) as HTMLButtonElement;
}

function card(effect: string) {
  return screen.getByRole("radio", { name: effect }).closest("label") as HTMLElement;
}

test("the_target_menu_offers_the_screen_actions_for_that_target", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);

  expect(itemLabels(await openMenu(await targetRow("Demo Graphics Card"), "Actions for Demo Graphics Card"))).toEqual([
    "Choose this target",
    "Stop effect",
    "Hardware modes…",
    "Leave this device to another program…",
  ]);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(itemLabels(await openMenu(await targetRow("Demo Light Bar"), "Actions for Demo Light Bar"))).toEqual([
    "Choose this target",
    "Hardware modes…",
    "Leave this device to another program…",
  ]);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(itemLabels(await openMenu(await targetRow("Demo Memory Module", "Whole device", 1), "Actions for Demo Memory Module"))).toEqual([
    "Choose this target",
    "Hand back to PrismForge",
    "Hardware modes…",
  ]);
  const leave = within(screen.getByRole("menu")).queryByRole("menuitem", { name: /Leave/ });
  expect(leave).toBeNull();
});

test("the_leave_item_is_last_after_a_separator_in_the_danger_style", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);

  const menu = await openMenu(await targetRow("Demo Graphics Card"), "Actions for Demo Graphics Card");
  const leave = within(menu).getByRole("menuitem", { name: "Leave this device to another program…" });

  expect(leave.className).toMatch(/danger/);
  expect(leave.previousElementSibling?.getAttribute("role")).toBe("separator");
});

test("choose_this_target_selects_it_like_its_row", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  const row = await targetRow("Demo Light Bar");

  await pick(row, "Actions for Demo Light Bar", "Choose this target");

  expect(await screen.findByRole("heading", { level: 2, name: "Demo Light Bar" })).toBeTruthy();
  expect(row.getAttribute("aria-current")).toBe("true");
});

test("stop_effect_in_the_target_menu_stops_that_target_with_the_same_command_as_its_button", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ stop_effect: () => stopRestored }));
  render(App);
  await fireEvent.click(await targetRow("Demo Light Bar"));

  await pick(await targetRow("Demo Graphics Card"), "Actions for Demo Graphics Card", "Stop effect");

  expect(await screen.findByRole("heading", { level: 2, name: "Demo Graphics Card" })).toBeTruthy();
  await waitFor(() => expect(args(shell, "stop_effect")).toEqual([{ target: { device: "demo-graphics-card", zone: null } }]));
  await waitFor(() => expect(screen.getByRole("button", { name: "Stop effect" })).toBeTruthy());
  await fireEvent.click(screen.getByRole("button", { name: "Stop effect" }));
  await waitFor(() => expect(args(shell, "stop_effect")).toHaveLength(2));
  expect(args(shell, "stop_effect")[1]).toEqual(args(shell, "stop_effect")[0]);
});

test("hand_back_in_the_target_menu_calls_the_same_command_as_its_button", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ clear_external: () => null }));
  render(App);

  await pick(await targetRow("Demo Memory Module", "Whole device", 1), "Actions for Demo Memory Module", "Hand back to PrismForge");

  await waitFor(() => expect(args(shell, "clear_external")).toEqual([{ device: "demo-memory-2" }]));
});

test("hardware_modes_and_leave_open_their_disclosure_and_send_nothing", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  const before = shell.calls.length;

  await pick(await targetRow("Demo Light Bar"), "Actions for Demo Light Bar", "Hardware modes…");
  const modes = await screen.findByText("Hardware modes, which run on the device itself");
  await waitFor(() => expect(document.activeElement).toBe(modes));
  expect((modes.closest("details") as HTMLDetailsElement).open).toBe(true);

  await pick(await targetRow("Demo Light Bar"), "Actions for Demo Light Bar", "Leave this device to another program…");
  const field = await screen.findByRole("textbox", { name: "Program name (optional)" });
  await waitFor(() => expect(document.activeElement).toBe(field));
  expect(shell.calls.slice(before).map((call) => call.command)).not.toContain("declare_external");
  expect(shell.calls.slice(before).map((call) => call.command)).not.toContain("select_mode");
});

test("the_effect_card_menu_chooses_and_applies_with_the_same_command_as_the_apply_button", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await fireEvent.click(await targetRow("Demo Graphics Card"));

  const menu = await openMenu(card("Gradient"), "Actions for Gradient");
  expect(itemLabels(menu)).toEqual(["Choose this effect", "Apply to Demo Graphics Card"]);
  await fireEvent.click(within(menu).getByRole("menuitem", { name: "Choose this effect" }));
  expect((screen.getByRole("radio", { name: "Gradient" }) as HTMLInputElement).checked).toBe(true);
  expect(args(shell, "apply_effect")).toEqual([]);

  await pick(card("Solid colour"), "Actions for Solid colour", "Apply to Demo Graphics Card");
  await waitFor(() => expect(args(shell, "apply_effect")).toHaveLength(1));
  expect((screen.getByRole("radio", { name: "Solid colour" }) as HTMLInputElement).checked).toBe(true);

  await fireEvent.click(screen.getByRole("button", { name: "Apply effect" }));
  await waitFor(() => expect(args(shell, "apply_effect")).toHaveLength(2));
  expect(args(shell, "apply_effect")[1]).toEqual(args(shell, "apply_effect")[0]);
});

test("apply_in_the_effect_card_menu_is_off_with_the_same_reason_as_the_bar", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await fireEvent.click(await targetRow("Demo Light Bar"));

  const menu = await openMenu(card("Gradient"), "Actions for Gradient");
  const apply = within(menu).getByRole("menuitem", { name: "Apply to Demo Light Bar" });

  expect(apply.getAttribute("aria-disabled")).toBe("true");
  expect(apply.textContent).toMatch(/This device has no Direct Mode, so PrismForge effects cannot run on it\./);
  await fireEvent.click(apply);
  expect(args(shell, "apply_effect")).toEqual([]);
});
