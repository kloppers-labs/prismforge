import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { groupsDemo } from "../../test/fixtures/groups";
import { demoConnected } from "../../test/fixtures/sessions";
import { itemLabels, openMenu, pick } from "../../test/menus";
import { componentRow, inspector, openSetup, setupHandlers } from "../../test/setup";

function turn(): string {
  return (within(inspector()).getByRole("spinbutton", { name: "Turn" }) as HTMLInputElement).value;
}

test("a_component_row_menu_offers_the_editor_actions_with_the_destructive_one_last", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  const menu = await openMenu(componentRow(list, "Graphics card"), "Actions for Graphics card");

  expect(itemLabels(menu)).toEqual([
    "Select only this component",
    "Turn right 15°",
    "Flip",
    "Identify",
    "Make a group from the selection…",
    "Take out of the PC view",
    "Delete component…",
  ]);
  expect(within(menu).getAllByRole("separator")).toHaveLength(1);
  expect(document.activeElement?.textContent).toContain("Select only this component");
});

test("a_menu_item_selects_its_row_and_runs_the_same_edit_as_the_button", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  await pick(componentRow(list, "Graphics card"), "Actions for Graphics card", "Turn right 15°");

  expect(componentRow(list, "Graphics card").getAttribute("aria-selected")).toBe("true");
  expect(turn()).toBe("15");
  expect(screen.getByRole("button", { name: "Undo" })).toBeTruthy();
});

test("delete_from_the_menu_asks_the_same_question_and_focuses_keep", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  await pick(componentRow(list, "Graphics card"), "Actions for Graphics card", "Delete component…");

  const dialog = await screen.findByRole("dialog", { name: "Delete Graphics card?" });
  await waitFor(() => expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Keep component" })));
  expect(componentRow(list, "Graphics card")).toBeTruthy();
});

test("the_drawing_opens_the_menu_of_the_component_under_the_pointer", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  await openSetup();

  await fireEvent.contextMenu(document.querySelector('[data-component="c-front-fan-1"] .led')!);

  expect(screen.getByRole("menu", { name: "Actions for Front fan 1" })).toBeTruthy();
});

test("identify_lights_the_whole_zone_and_says_which_components_light_with_it", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { identify: () => new Promise(() => {}) }));
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Front fan 2"));

  await fireEvent.click(within(inspector()).getByRole("button", { name: "Identify" }));

  expect(shell.calls.find((call) => call.command === "identify")!.args).toEqual({ device: "demo-motherboard", zone: 1 });
  expect(
    await screen.findByText(/^Identify lights all of Addressable Header 1 on Demo Motherboard, so Front fan 1, Front fan 2, Front fan 3 and Rear fan light together\./),
  ).toBeTruthy();
});

test("a_saved_group_is_made_from_the_outputs_of_the_selected_components", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { create_group: () => groupsDemo }));
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Front fan 1"));
  await fireEvent.click(componentRow(list, "Rear fan"), { ctrlKey: true });
  await fireEvent.click(componentRow(list, "Graphics card"), { ctrlKey: true });

  await fireEvent.click(within(inspector()).getByRole("button", { name: "Make a group from the selection…" }));
  const dialog = await screen.findByRole("dialog", { name: "New group" });
  await fireEvent.input(within(dialog).getByRole("textbox", { name: "Group name" }), { target: { value: "Case lights" } });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Make group" }));

  await waitFor(() => expect(shell.commands()).toContain("create_group"));
  expect(shell.calls.find((call) => call.command === "create_group")!.args).toEqual({
    name: "Case lights",
    targets: [
      { device: "demo-motherboard", zone: 1 },
      { device: "demo-graphics-card", zone: null },
    ],
  });
});

test("a_saved_group_selects_the_components_of_its_members", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { groups: () => groupsDemo }));
  render(App);
  const list = await openSetup();
  const groups = screen.getByRole("region", { name: "Saved groups" });

  await fireEvent.click(within(within(groups).getByRole("group", { name: "Memory" })).getByRole("button", { name: "Select its components" }));

  expect(
    within(list)
      .getAllByRole("option")
      .filter((row) => row.getAttribute("aria-selected") === "true")
      .map((row) => row.querySelector(".pick-name")?.textContent),
  ).toEqual(["Memory module 1", "Memory module 2"]);
  expect(groups.textContent).toContain("Selected Memory: 2 components.");
});
