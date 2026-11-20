import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { StoreView } from "../../lib/api/bindings/StoreView";
import { fakeShell } from "../../test/backend";
import { effectHandlers, wait } from "../../test/effects";
import { appliedFully } from "../../test/fixtures/applies";
import { detailsEvening, detailsImported, detailsOffline } from "../../test/fixtures/profileDetails";
import { previewChanges } from "../../test/fixtures/saves";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeOffline, storeTwo, storeUnsaved } from "../../test/fixtures/stores";
import { itemLabels, openMenu, pick } from "../../test/menus";

const EVENING = "p-0000000000000001";
const OLDER = "p-0000000000000002";

function handlers(store: StoreView, extra: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
  return effectHandlers({
    profile_store: () => store,
    profile_details: (args) => (args.id === OLDER ? detailsImported : detailsEvening),
    ...extra,
  });
}

function args(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command).map((call) => call.args);
}

async function row(name: string) {
  const screens = await screen.findByRole("navigation", { name: "Main" });
  if (!screen.queryByRole("navigation", { name: "Profiles" })) {
    await fireEvent.click(within(screens).getByRole("button", { name: "Profiles" }));
  }
  const list = await screen.findByRole("navigation", { name: "Profiles" });
  return (await within(list).findByRole("button", { name: new RegExp(`^${name.replace(/[()]/g, "\\$&")} \\d`) })) as HTMLButtonElement;
}

test("the_profile_row_menu_offers_the_actions_of_that_profile", async () => {
  fakeShell(demoConnected, handlers(storeTwo));
  render(App);

  const active = await openMenu(await row("Evening"), "Actions for Evening");
  expect(itemLabels(active)).toEqual([
    "Apply profile",
    "Save changes to Evening",
    "Rename",
    "Duplicate",
    "Export…",
    "Delete",
  ]);
  expect(within(active).queryByRole("menuitemcheckbox")).toBeNull();
  expect(within(active).getByRole("menuitem", { name: "Delete" }).previousElementSibling?.getAttribute("role")).toBe("separator");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });

  expect(itemLabels(await openMenu(await row("Evening (2)"), "Actions for Evening (2)"))).not.toContain("Save changes to Evening (2)");
});

test("apply_in_the_menu_calls_the_same_command_as_the_apply_button", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { apply_profile: () => appliedFully }));
  render(App);

  await pick(await row("Evening (2)"), "Actions for Evening (2)", "Apply profile");

  const article = await screen.findByRole("article", { name: "Evening (2)" });
  await waitFor(() => expect(args(shell, "apply_profile")).toEqual([{ id: OLDER }]));
  await fireEvent.click(await within(article).findByRole("button", { name: "Apply profile" }));
  await waitFor(() => expect(args(shell, "apply_profile")).toEqual([{ id: OLDER }, { id: OLDER }]));
});

test("delete_in_the_menu_asks_first_and_returns_focus_to_the_row", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { delete_profile: () => null }));
  render(App);
  const older = await row("Evening (2)");

  await pick(older, "Actions for Evening (2)", "Delete");
  const dialog = await screen.findByRole("dialog", { name: "Delete Evening (2)?" });
  expect(within(dialog).getByText(/The profile file is removed from this computer\./)).toBeTruthy();
  expect(args(shell, "delete_profile")).toEqual([]);
  await fireEvent.keyDown(dialog, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(document.activeElement).toBe(older);

  await pick(older, "Actions for Evening (2)", "Delete");
  await fireEvent.click(within(await screen.findByRole("dialog")).getByRole("button", { name: "Delete profile" }));
  expect(args(shell, "delete_profile")).toEqual([{ id: OLDER }]);
});

test("export_in_the_menu_says_what_the_file_holds_before_a_destination_is_chosen", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { export_profile: () => ({ omitted_fields: 0 }) }));
  render(App);

  await pick(await row("Evening"), "Actions for Evening", "Export…");
  const dialog = await screen.findByRole("dialog", { name: "Export Evening" });
  expect(within(dialog).getByText(/It leaves out serial numbers, device locations/)).toBeTruthy();
  expect(args(shell, "export_profile")).toEqual([]);
  await fireEvent.click(within(dialog).getByRole("button", { name: "Choose where to save" }));

  expect(args(shell, "export_profile")).toEqual([{ id: EVENING }]);
});

test("rename_and_duplicate_in_the_menu_use_the_same_paths_as_their_buttons", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { duplicate_profile: () => OLDER }));
  render(App);

  await pick(await row("Evening"), "Actions for Evening", "Rename");
  const field = await screen.findByRole("textbox", { name: "New name" });
  await waitFor(() => expect(document.activeElement).toBe(field));
  expect((field as HTMLInputElement).value).toBe("Evening");
  await fireEvent.keyDown(field, { key: "Escape" });

  await pick(await row("Evening"), "Actions for Evening", "Duplicate");
  await waitFor(() => expect(args(shell, "duplicate_profile")).toEqual([{ id: EVENING }]));
});

test("save_changes_in_the_menu_shows_the_change_list_first", async () => {
  const shell = fakeShell(demoConnected, handlers(storeUnsaved, { preview_profile_update: () => previewChanges }));
  render(App);

  await pick(await row("Evening"), "Actions for Evening", "Save changes to Evening");

  expect(await screen.findByRole("dialog", { name: "Save changes to Evening?" })).toBeTruthy();
  expect(args(shell, "preview_profile_update")).toEqual([{ id: EVENING }]);
  expect(args(shell, "update_profile")).toEqual([]);
});

test("apply_that_is_off_stays_in_the_menu_with_the_article's_reason", async () => {
  const shell = fakeShell(null, { profile_store: () => storeOffline, profile_details: () => detailsOffline });
  render(App);
  const desk = await row("Desk");
  await fireEvent.click(desk);
  const article = await screen.findByRole("article", { name: "Desk" });
  const off = within(article).getByText(/^Apply is off\./).textContent ?? "";

  const menu = await openMenu(desk, "Actions for Desk");
  const apply = within(menu).getByRole("menuitem", { name: "Apply profile" });

  expect(apply.getAttribute("aria-disabled")).toBe("true");
  const reason = document.getElementById(apply.getAttribute("aria-describedby") ?? "")?.textContent ?? "";
  expect(off).toBe(`Apply is off. ${reason}`);
  expect(document.activeElement).not.toBe(apply);
  await fireEvent.click(apply);
  expect(args(shell, "apply_profile")).toEqual([]);
});

test("a_menu_request_is_dropped_when_another_row_is_chosen_before_the_details_load", async () => {
  let release: () => void = () => {};
  const later = new Promise<void>((resolve) => (release = resolve));
  let slow = true;
  const shell = fakeShell(
    demoConnected,
    handlers(storeTwo, {
      delete_profile: () => null,
      profile_details: async (call) => {
        if (call.id === OLDER && slow) {
          await later;
        }
        return call.id === OLDER ? detailsImported : detailsEvening;
      },
    }),
  );
  render(App);

  await pick(await row("Evening (2)"), "Actions for Evening (2)", "Delete");
  await fireEvent.click(await row("Evening"));
  await screen.findByRole("article", { name: "Evening" });
  slow = false;
  release();
  await wait(50);
  expect(screen.queryByRole("dialog")).toBeNull();

  await fireEvent.click(await row("Evening (2)"));
  await screen.findByRole("article", { name: "Evening (2)" });
  await wait(50);
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(args(shell, "delete_profile")).toEqual([]);
});

test("a_menu_request_is_dropped_when_the_details_fail_to_load", async () => {
  let fail = true;
  fakeShell(
    demoConnected,
    handlers(storeTwo, {
      profile_details: (call) => {
        if (call.id === OLDER && fail) {
          return Promise.reject({ Session: "Busy" });
        }
        return call.id === OLDER ? detailsImported : detailsEvening;
      },
    }),
  );
  render(App);

  await pick(await row("Evening (2)"), "Actions for Evening (2)", "Export…");
  await wait(50);
  expect(screen.queryByRole("dialog")).toBeNull();
  fail = false;

  await fireEvent.click(await row("Evening"));
  await screen.findByRole("article", { name: "Evening" });
  await fireEvent.click(await row("Evening (2)"));
  await screen.findByRole("article", { name: "Evening (2)" });
  await wait(50);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("duplicate_in_the_menu_shows_its_profile_first_like_every_row_item", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { duplicate_profile: () => OLDER }));
  render(App);

  await fireEvent.click(await row("Evening"));
  await screen.findByRole("article", { name: "Evening" });
  const before = shell.calls.length;
  await pick(await row("Evening (2)"), "Actions for Evening (2)", "Duplicate");
  await waitFor(() => expect(args(shell, "duplicate_profile")).toEqual([{ id: OLDER }]));
  const later = shell.calls.slice(before).map((call) => `${call.command}:${String(call.args.id)}`);
  expect(later.indexOf(`profile_details:${OLDER}`)).toBeGreaterThan(-1);
  expect(later.indexOf(`profile_details:${OLDER}`)).toBeLessThan(later.indexOf(`duplicate_profile:${OLDER}`));
});
