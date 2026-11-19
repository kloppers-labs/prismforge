import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell, type Call } from "../../test/backend";
import { button, effectHandlers, openEffects } from "../../test/effects";
import { groupsDemo, groupsEmpty, groupsRealMissing } from "../../test/fixtures/groups";
import { demoConnected, openRgbShelfGone } from "../../test/fixtures/sessions";
import { openMenu, pick } from "../../test/menus";

function args(calls: Call[], command: string) {
  return calls.filter((call) => call.command === command).map((call) => call.args);
}

function groupRow(name: string): HTMLElement {
  const panel = screen.getByRole("region", { name: "Groups" });
  return within(panel).getByText(name, { selector: ".name" }).closest("li") as HTMLElement;
}

function option(device: string, part = "Whole device", index = 0): HTMLElement {
  const targets = screen.getByRole("listbox", { name: "Targets" });
  const group = within(targets).getAllByRole("group", { name: device })[index]!;
  return within(group).getByRole("option", { name: new RegExp(`^${part}`) });
}

test("a_group_selects_its_members_and_names_each_member_missing_today", async () => {
  fakeShell(openRgbShelfGone, effectHandlers({ groups: () => groupsRealMissing }));
  render(App);
  await openEffects();

  const row = await waitFor(() => groupRow("Shelf and desk"));
  expect(within(row).getByText("2 targets, 1 not here today")).toBeTruthy();
  await fireEvent.click(within(row).getByRole("button", { name: "Select" }));

  expect(screen.getByText("1 target selected")).toBeTruthy();
  expect(screen.getByText("Selected Shelf and desk: 1 target. Left out: Fixture Shelf Strip (not found today).")).toBeTruthy();
  expect(option("Fixture Desk Strip").getAttribute("aria-selected")).toBe("true");
  expect(within(row).getByText("Fixture Shelf Strip: not found today")).toBeTruthy();
});

test("everything_selects_every_target_that_can_take_an_effect", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await openEffects();

  await fireEvent.click(within(await waitFor(() => groupRow("Everything"))).getByRole("button", { name: "Select" }));

  expect(screen.getByText("4 targets selected")).toBeTruthy();
  expect(option("Demo Light Bar").getAttribute("aria-selected")).toBe("false");
});

test("a_new_group_holds_the_selected_targets", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ create_group: () => groupsDemo }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"), { ctrlKey: true });
  await fireEvent.click(option("Demo Motherboard", "Addressable Header 1"), { ctrlKey: true });

  await fireEvent.click(button("New group…"));
  const dialog = screen.getByRole("dialog", { name: "New group" });
  await fireEvent.input(within(dialog).getByRole("textbox", { name: "Group name" }), { target: { value: "Desk" } });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Create group" }));

  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(args(shell.calls, "create_group")).toEqual([
    {
      name: "Desk",
      targets: [
        { device: "demo-graphics-card", zone: null },
        { device: "demo-motherboard", zone: 1 },
      ],
    },
  ]);
  expect(groupRow("Memory")).toBeTruthy();
});

test("a_refused_group_name_is_shown_in_the_dialog_and_nothing_closes", async () => {
  fakeShell(demoConnected, effectHandlers({ create_group: () => Promise.reject({ Store: { GroupNameTaken: { name: "Desk" } } }) }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Graphics Card"));

  await fireEvent.click(button("New group…"));
  const dialog = screen.getByRole("dialog", { name: "New group" });
  await fireEvent.input(within(dialog).getByRole("textbox", { name: "Group name" }), { target: { value: "Desk" } });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Create group" }));

  expect(await within(dialog).findByRole("alert")).toHaveProperty("textContent", "Another group is already called Desk. Choose another name.");
});

test("add_to_group_on_a_target_row_adds_that_target_through_the_dialog", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ groups: () => groupsDemo, add_group_members: () => groupsDemo }));
  render(App);
  await openEffects();
  await waitFor(() => groupRow("Desk"));

  await pick(option("Demo Light Bar"), "Actions for Demo Light Bar", "Add to group…");
  const dialog = screen.getByRole("dialog", { name: "Add to a group" });
  await fireEvent.click(within(dialog).getByRole("radio", { name: "Memory" }));
  await fireEvent.click(within(dialog).getByRole("button", { name: "Add to group" }));

  await waitFor(() => expect(args(shell.calls, "add_group_members")).toHaveLength(1));
  const memory = groupsDemo.groups.find((group) => group.name === "Memory")!;
  expect(args(shell.calls, "add_group_members")[0]).toEqual({ id: memory.id, targets: [{ device: "demo-light-bar", zone: null }] });
});

test("rename_and_delete_in_the_group_menu_open_the_same_dialogs_as_their_buttons", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ groups: () => groupsDemo, rename_group: () => groupsDemo, delete_group: () => groupsEmpty }));
  render(App);
  await openEffects();
  const desk = groupsDemo.groups.find((group) => group.name === "Desk")!;

  await pick(await waitFor(() => groupRow("Desk")), "Actions for Desk", "Rename…");
  const rename = screen.getByRole("dialog", { name: "Rename Desk" });
  const field = within(rename).getByRole("textbox", { name: "Group name" }) as HTMLInputElement;
  expect(field.value).toBe("Desk");
  await fireEvent.input(field, { target: { value: "Desk lights" } });
  await fireEvent.click(within(rename).getByRole("button", { name: "Rename group" }));
  await waitFor(() => expect(args(shell.calls, "rename_group")).toEqual([{ id: desk.id, name: "Desk lights" }]));

  await pick(groupRow("Desk"), "Actions for Desk", "Delete…");
  const confirm = screen.getByRole("dialog", { name: "Delete Desk?" });
  expect(document.activeElement).toBe(within(confirm).getByRole("button", { name: "Keep group" }));
  await fireEvent.click(within(confirm).getByRole("button", { name: "Delete group" }));

  await waitFor(() => expect(args(shell.calls, "delete_group")).toEqual([{ id: desk.id }]));
  await waitFor(() => expect(screen.queryByText("Desk", { selector: ".name" })).toBeNull());
});

test("the_group_menu_applies_the_chosen_effect_to_the_group", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ groups: () => groupsDemo, apply_effect_batch: () => ({ results: [] }) }));
  render(App);
  await openEffects();
  await fireEvent.click(option("Demo Light Bar"));
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await pick(await waitFor(() => groupRow("Desk")), "Actions for Desk", "Apply Rainbow wave to group");

  await waitFor(() => expect(args(shell.calls, "apply_effect_batch")).toHaveLength(1));
  expect(args(shell.calls, "apply_effect_batch")[0]?.targets).toEqual([
    { device: "demo-graphics-card", zone: null },
    { device: "demo-motherboard", zone: 1 },
  ]);
});

test("a_member_can_be_removed_and_the_delete_item_sits_apart", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ groups: () => groupsDemo, remove_group_member: () => groupsDemo }));
  render(App);
  await openEffects();
  const row = await waitFor(() => groupRow("Desk"));

  const menu = await openMenu(row, "Actions for Desk");
  const remove = within(menu).getByRole("menuitem", { name: "Delete…" });
  expect(remove.className).toMatch(/danger/);
  expect(remove.previousElementSibling?.getAttribute("role")).toBe("separator");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });

  await fireEvent.click(within(row).getByText("Members and actions"));
  await fireEvent.click(within(row).getByRole("button", { name: "Remove Demo Graphics Card" }));

  await waitFor(() => expect(args(shell.calls, "remove_group_member")).toHaveLength(1));
  expect(args(shell.calls, "remove_group_member")[0]?.target).toEqual({ device: "demo-graphics-card", zone: null });
});

test("a_recovered_groups_file_says_so_outside_any_disclosure", async () => {
  fakeShell(demoConnected, effectHandlers({
    groups: () => ({ ...groupsEmpty, notices: [{ file: "groups.json", event: { RestoredFromBackup: { problem: null, set_aside: null, rewritten: true } } }] }),
  }));
  render(App);
  await openEffects();

  const text = await screen.findByText("groups.json was missing. It used the backup, so your last change to a group may be missing.");
  expect(text.closest("details")).toBeNull();
});
