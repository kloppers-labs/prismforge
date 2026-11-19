import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { StoreView } from "../../lib/api/bindings/StoreView";
import { fakeShell, refuse } from "../../test/backend";
import { effectHandlers, wait } from "../../test/effects";
import { appliedFully, appliedPartly, waitingForConnection } from "../../test/fixtures/applies";
import * as errors from "../../test/fixtures/errors";
import { importedRenamed } from "../../test/fixtures/imports";
import { detailsEvening, detailsImported, detailsOffline } from "../../test/fixtures/profileDetails";
import { previewChanges, previewNone, savedChanges, savedNew } from "../../test/fixtures/saves";
import { demoConnected } from "../../test/fixtures/sessions";
import {
  storeEmpty,
  storeOffline,
  storeRecovered,
  storeSaved,
  storeTwo,
  storeUnsaved,
} from "../../test/fixtures/stores";

const EVENING = "p-0000000000000001";
const OLDER = "p-0000000000000002";

function details(args: Record<string, unknown>) {
  return args.id === OLDER ? detailsImported : detailsEvening;
}

function handlers(store: StoreView, extra: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
  return effectHandlers({ profile_store: () => store, profile_details: details, ...extra });
}

function called(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command);
}

async function openProfiles() {
  const screens = await screen.findByRole("navigation", { name: "Screens" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Profiles" }));
  return screen.findByRole("navigation", { name: "Profiles" });
}

async function chooseProfile(name: string) {
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: new RegExp(`^${name.replace(/[()]/g, "\\$&")} \\d`) }));
  return screen.findByRole("article", { name });
}

function result() {
  return screen.getByRole("status", { name: "Profile result" });
}

test("lists_each_profile_with_its_contents_and_how_many_devices_are_found_today", async () => {
  fakeShell(demoConnected, handlers(storeTwo));
  render(App);
  const list = await openProfiles();

  const evening = within(list).getByRole("button", { name: /^Evening\b(?! \()/ });
  expect(evening.textContent).toMatch(/2 effects, 1 hardware mode, 1 device for another program/);
  expect(evening.textContent).toMatch(/All 5 devices found\./);
  expect(evening.textContent).toMatch(/Active in this session/);
  const older = within(list).getByRole("button", { name: /^Evening \(2\)/ });
  expect(older.textContent).toMatch(/3 of 5 devices found\. 2 not found today\./);
  expect(older.textContent).not.toMatch(/Active in this session/);
});

test("details_name_every_setting_and_why_it_would_be_skipped_today", async () => {
  const missing = structuredClone(detailsImported);
  const board = missing.settings.find((setting) => setting.label === "Demo Motherboard");
  if (board) {
    board.today = { Skipped: { reason: "Missing" } };
  }
  fakeShell(demoConnected, handlers(storeTwo, { profile_details: (args) => (args.id === OLDER ? missing : detailsEvening) }));
  render(App);
  const article = await chooseProfile("Evening (2)");

  const settings = within(article).getByRole("list", { name: "What this profile holds" });
  const items = within(settings).getAllByRole("listitem").map((item) => item.textContent ?? "");
  expect(items).toHaveLength(4);
  expect(items.some((item) => /Another program: OpenRGB Effects plugin/.test(item) && /Skipped today\. Identical devices/.test(item))).toBe(true);
  expect(items.some((item) => /Hardware mode Pulse/.test(item) && /mode list changed/.test(item) && /Breathing/.test(item))).toBe(true);
  expect(items.some((item) => /aurora/.test(item) && /does not have this effect/.test(item))).toBe(true);
  expect(items.some((item) => /Demo Motherboard/.test(item) && /Not found\./.test(item))).toBe(true);
  expect(within(article).getByText(/Unresolved settings stay in the profile/)).toBeTruthy();
});

test("update_shows_the_change_list_first_and_saves_only_after_confirmation", async () => {
  const shell = fakeShell(
    demoConnected,
    handlers(storeUnsaved, { preview_profile_update: () => previewChanges, update_profile: () => savedChanges }),
  );
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Save changes to Evening" }));
  const dialog = await screen.findByRole("dialog", { name: "Save changes to Evening?" });
  expect(within(dialog).getByText(/replaces Rainbow wave on the whole device with Moving highlight on the whole device/)).toBeTruthy();
  expect(within(dialog).getByText(/removes Rainbow wave on the whole device/)).toBeTruthy();
  expect(within(dialog).getByText(/adds Solid colour on Addressable Header 1/)).toBeTruthy();
  expect(within(dialog).getByText(/Frame rate: from 30 to 45/)).toBeTruthy();
  expect(within(dialog).getByText(/Nothing is saved to the devices/)).toBeTruthy();
  await fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(called(shell, "update_profile")).toHaveLength(0);
  expect(document.activeElement).toBe(within(article).getByRole("button", { name: "Save changes to Evening" }));

  await fireEvent.click(within(article).getByRole("button", { name: "Save changes to Evening" }));
  const again = await screen.findByRole("dialog", { name: "Save changes to Evening?" });
  await fireEvent.click(within(again).getByRole("button", { name: "Save changes" }));

  expect(called(shell, "update_profile").map((call) => call.args)).toEqual([{ id: EVENING }]);
  await waitFor(() => expect(result().textContent).toMatch(/Saved 4 changes to Evening on this computer\./));
});

test("an_update_with_nothing_new_says_so_and_saves_nothing", async () => {
  const shell = fakeShell(demoConnected, handlers(storeSaved, { preview_profile_update: () => previewNone }));
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Save changes to Evening" }));

  await waitFor(() => expect(result().textContent).toMatch(/Nothing to save\./));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(called(shell, "update_profile")).toHaveLength(0);
});

test("not_active_offers_save_as_new_and_explains_it", async () => {
  const store = structuredClone(storeTwo);
  store.summary.active = OLDER;
  fakeShell(demoConnected, handlers(store, { preview_profile_update: refuse(errors.notActive) }));
  render(App);
  const article = await chooseProfile("Evening (2)");

  await fireEvent.click(within(article).getByRole("button", { name: "Save changes to Evening (2)" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening \(2\) is not the profile applied in this session/));
  expect(result().getAttribute("class")).not.toMatch(/problem/);
  await fireEvent.click(within(result()).getByRole("button", { name: "Save as a new profile" }));
  expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Profile name" }));
});

test("apply_lists_each_target_and_a_partial_apply_is_not_called_complete", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { apply_profile: (args) => (args.id === OLDER ? appliedPartly : appliedFully) }));
  render(App);
  const older = await chooseProfile("Evening (2)");

  await fireEvent.click(within(older).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening \(2\) is applied in part\. 3 of 4 settings were skipped and stay in the profile\./));
  expect(result().textContent).not.toMatch(/All \d+ settings were accepted/);
  const outcomes = within(result()).getAllByRole("listitem").map((item) => item.textContent ?? "");
  expect(outcomes).toHaveLength(4);
  expect(outcomes.filter((item) => /Skipped\./.test(item))).toHaveLength(3);
  expect(outcomes.filter((item) => /Applied\./.test(item))).toHaveLength(1);

  const evening = await chooseProfile("Evening");
  await fireEvent.click(within(evening).getByRole("button", { name: "Apply profile" }));
  await waitFor(() => expect(result().textContent).toMatch(/Evening is applied\. All 4 settings were accepted\./));
  expect(called(shell, "apply_profile").map((call) => call.args)).toEqual([{ id: OLDER }, { id: EVENING }]);
});

test("a_marked_profile_that_waits_for_a_connection_says_so", async () => {
  fakeShell(demoConnected, handlers(storeTwo, { apply_profile: () => waitingForConnection }));
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening waits for a connection\. It applies once when PrismForge connects\./));
});

test("delete_asks_first_and_says_the_file_leaves_this_computer", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { delete_profile: () => null }));
  render(App);
  const article = await chooseProfile("Evening");
  const remove = within(article).getByRole("button", { name: "Delete" });

  await fireEvent.click(remove);
  const dialog = await screen.findByRole("dialog", { name: "Delete Evening?" });
  expect(within(dialog).getByText(/The profile file is removed from this computer\./)).toBeTruthy();
  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Keep profile" }));
  await fireEvent.keyDown(dialog, { key: "Escape" });
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(document.activeElement).toBe(remove);
  expect(called(shell, "delete_profile")).toHaveLength(0);

  await fireEvent.click(remove);
  await fireEvent.click(within(await screen.findByRole("dialog")).getByRole("button", { name: "Delete profile" }));

  expect(called(shell, "delete_profile").map((call) => call.args)).toEqual([{ id: EVENING }]);
});

test("the_confirmation_dialog_keeps_focus_inside", async () => {
  fakeShell(demoConnected, handlers(storeTwo));
  render(App);
  const article = await chooseProfile("Evening");
  await fireEvent.click(within(article).getByRole("button", { name: "Delete" }));
  const dialog = await screen.findByRole("dialog", { name: "Delete Evening?" });
  const [first, last] = [
    within(dialog).getByRole("button", { name: "Keep profile" }),
    within(dialog).getByRole("button", { name: "Delete profile" }),
  ];

  last.focus();
  await fireEvent.keyDown(dialog, { key: "Tab" });
  expect(document.activeElement).toBe(first);
  await fireEvent.keyDown(dialog, { key: "Tab", shiftKey: true });
  expect(document.activeElement).toBe(last);
});

test("import_shows_its_report_and_never_replaces_a_profile", async () => {
  const shell = fakeShell(demoConnected, handlers(storeSaved, { import_profile: () => importedRenamed }));
  render(App);
  await openProfiles();

  await fireEvent.click(screen.getByRole("button", { name: "Import a profile" }));

  const report = await screen.findByRole("status", { name: "Import result" });
  await waitFor(() => expect(report.textContent).toMatch(/Imported as Evening \(2\)\./));
  expect(report.textContent).toMatch(/Another profile was already called Evening, so the import got a new name\. Nothing was replaced\./);
  expect(report.textContent).toMatch(/3 of 5 devices found today\./);
  expect(report.textContent).toMatch(/1 effect is not available in this version\. It stays in the profile\./);
  expect(report.textContent).toMatch(/Importing writes nothing to your devices\./);
  expect(called(shell, "import_profile")[0]?.args).toEqual({});
});

test("export_states_what_the_file_holds_before_a_destination_is_chosen", async () => {
  const shell = fakeShell(demoConnected, handlers(storeSaved, { export_profile: () => ({ omitted_fields: 0 }) }));
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Export" }));
  const dialog = await screen.findByRole("dialog", { name: "Export Evening" });
  expect(dialog.textContent).toMatch(
    /The file holds the profile name, the model names of its devices and any names you typed\. It holds no serial numbers, device locations or other details that identify this computer\./,
  );
  expect(called(shell, "export_profile")).toHaveLength(0);
  await fireEvent.click(within(dialog).getByRole("button", { name: "Choose where to save" }));

  expect(called(shell, "export_profile").map((call) => call.args)).toEqual([{ id: EVENING }]);
  await waitFor(() => expect(result().textContent).toMatch(/Exported Evening\./));
});

test("apply_on_connect_is_off_by_default_and_its_text_is_exact", async () => {
  const shell = fakeShell(demoConnected, handlers(storeSaved, { set_apply_on_connect: () => null }));
  render(App);
  const article = await chooseProfile("Evening");

  const toggle = within(article).getByRole("switch", { name: "Apply this profile when PrismForge connects" });
  expect(toggle.getAttribute("aria-checked")).toBe("false");
  expect(toggle.getAttribute("aria-describedby")).toBeTruthy();
  const description = document.getElementById(toggle.getAttribute("aria-describedby") ?? "");
  expect(description?.textContent?.trim()).toBe(
    "When PrismForge connects, it applies this profile once. It does not start PrismForge or connect by itself.",
  );
  await fireEvent.click(toggle);

  expect(called(shell, "set_apply_on_connect").map((call) => call.args)).toEqual([{ id: EVENING }]);
});

test("recovery_notices_stay_until_dismissed_and_carry_the_instructions", async () => {
  let store = storeRecovered;
  const shell = fakeShell(demoConnected, {
    ...handlers(storeRecovered),
    profile_store: () => store,
    dismiss_store_notice: () => {
      store = { ...storeRecovered, summary: { ...storeRecovered.summary, notices: storeRecovered.summary.notices.slice(1) } };
      return true;
    },
  });
  render(App);
  const banner = await screen.findByRole("status", { name: "Repaired files" });
  expect(banner.textContent).toMatch(/PrismForge repaired stored files\./);
  await openProfiles();

  const notices = screen.getByRole("region", { name: "Repaired files" });
  expect(notices.textContent).toMatch(/could not read devices\.json or its backup/);
  expect(notices.textContent).toMatch(/Close PrismForge before you touch them/);
  expect(notices.textContent).toMatch(/used its backup/);
  expect(notices.textContent).toMatch(/Your last change to it may be missing/);
  await fireEvent.click(within(notices).getByRole("button", { name: "Dismiss the notice about devices.json" }));

  expect(called(shell, "dismiss_store_notice")[0]?.args).toEqual({ notice: storeRecovered.summary.notices[0] });
  await waitFor(() => expect(within(notices).queryByText(/could not read devices\.json/)).toBeNull());
  expect(notices.textContent).toMatch(/used its backup/);
});

test("the_header_shows_the_active_profile_and_unknown_unsaved_changes", async () => {
  const unknown = structuredClone(storeSaved);
  unknown.summary.unsaved_changes = null;
  let store: StoreView = storeUnsaved;
  fakeShell(demoConnected, handlers(storeUnsaved, { profile_store: () => store }));
  render(App);

  const status = await screen.findByRole("button", { name: /^Profile Evening/ });
  expect(status.textContent).toMatch(/4 unsaved changes/);
  store = unknown;
  await fireEvent.click(status);
  await waitFor(() => expect(screen.getByRole("button", { name: /^Profile Evening/ }).textContent).toMatch(/Unsaved changes unknown/));
  expect(screen.getByRole("navigation", { name: "Profiles" })).toBeTruthy();
});

test("busy_is_shown_and_not_retried", async () => {
  const shell = fakeShell(demoConnected, handlers(storeEmpty, { create_profile: refuse(errors.sessionBusy) }));
  render(App);
  await openProfiles();

  await fireEvent.input(screen.getByRole("textbox", { name: "Profile name" }), { target: { value: "Evening" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save as new profile" }));
  await waitFor(() => expect(result().textContent).toMatch(/still working on earlier requests/));
  await wait(400);

  expect(called(shell, "create_profile")).toHaveLength(1);
});

test("saving_the_current_lighting_names_the_new_profile", async () => {
  const shell = fakeShell(demoConnected, handlers(storeEmpty, { create_profile: () => savedNew }));
  render(App);
  await openProfiles();
  expect(screen.getByText(/No profiles yet\./)).toBeTruthy();
  expect(screen.getByText(/The new profile holds the effects, hardware modes, program statements and frame rate in use now\./)).toBeTruthy();

  await fireEvent.input(screen.getByRole("textbox", { name: "Profile name" }), { target: { value: "  Evening " } });
  await fireEvent.click(screen.getByRole("button", { name: "Save as new profile" }));

  expect(called(shell, "create_profile").map((call) => call.args)).toEqual([{ name: "Evening" }]);
  await waitFor(() => expect(result().textContent).toMatch(/Saved the current lighting as a new profile on this computer\./));
});

test("save_as_new_with_an_active_profile_says_what_it_holds", async () => {
  fakeShell(demoConnected, handlers(storeUnsaved));
  render(App);
  await openProfiles();

  expect(
    screen.getByText(
      "The new profile holds Evening and the changes you made since you applied or saved it. Effects that already ran before that, on devices Evening does not name, are not included.",
    ),
  ).toBeTruthy();
});

test("without_an_active_profile_save_changes_says_to_apply_or_save_first", async () => {
  const store = structuredClone(storeTwo);
  store.summary.active = null;
  fakeShell(demoConnected, handlers(store));
  render(App);
  const article = await chooseProfile("Evening");

  expect(within(article).queryByRole("button", { name: "Save changes to Evening" })).toBeNull();
  expect(within(article).getByText(/To save changes to this profile, apply it first\./)).toBeTruthy();
  expect(within(article).getByText(/until PrismForge closes/)).toBeTruthy();
});

test("demo_profiles_are_marked_as_demo_on_the_profiles_screen", async () => {
  fakeShell(demoConnected, handlers(storeSaved));
  render(App);
  await openProfiles();

  expect(screen.getByRole("complementary", { name: "Demo mode" })).toBeTruthy();
  expect(screen.getByText(/These are demo profiles\. They are kept apart from your real profiles and never apply to real hardware\./)).toBeTruthy();
});

test("not_connected_lists_what_still_works_and_turns_apply_and_save_off", async () => {
  const shell = fakeShell(null, { profile_store: () => storeOffline, profile_details: () => detailsOffline, rename_profile: () => null });
  render(App);
  const list = await openProfiles();
  expect(screen.getByText(/You can still rename, duplicate, delete, export and import profiles/)).toBeTruthy();
  await fireEvent.click(within(list).getByRole("button", { name: /^Desk/ }));
  const article = await screen.findByRole("article", { name: "Desk" });

  const apply = within(article).getByRole("button", { name: "Apply profile" }) as HTMLButtonElement;
  expect(apply.disabled).toBe(true);
  expect(within(article).getByText(/Apply is off\. PrismForge is not connected\./)).toBeTruthy();
  expect(within(article).getByText(/Not checked: PrismForge is not connected\./)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Save as new profile" })).toBeNull();
  expect(within(article).getByRole("switch", { name: "Apply this profile when PrismForge connects" }).getAttribute("aria-checked")).toBe("true");

  await fireEvent.click(within(article).getByRole("button", { name: "Rename" }));
  await fireEvent.input(within(article).getByRole("textbox", { name: "New name" }), { target: { value: "Desk lights" } });
  await fireEvent.click(within(article).getByRole("button", { name: "Save name" }));
  expect(called(shell, "rename_profile").map((call) => call.args)).toEqual([{ id: "p-0000000000000003", name: "Desk lights" }]);
});

test("another_instance_holding_the_store_is_explained", async () => {
  fakeShell(demoConnected, effectHandlers({ profile_store: refuse({ StoreUnavailable: "InUse" }) }));
  render(App);
  await openProfiles();

  expect(await screen.findByText(/Another PrismForge window is using the profile folder\./)).toBeTruthy();
});
