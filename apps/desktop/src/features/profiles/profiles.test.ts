import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { StoreView } from "../../lib/api/bindings/StoreView";
import { fakeShell, refuse } from "../../test/backend";
import { connectionAction, expandNotices, findDemoMarker, openProfileChip } from "../../test/shell";
import { effectHandlers, wait } from "../../test/effects";
import { appliedFully, appliedPartly, waitingForConnection } from "../../test/fixtures/applies";
import * as errors from "../../test/fixtures/errors";
import { importedRenamed } from "../../test/fixtures/imports";
import { detailsEvening, detailsImported, detailsOffline } from "../../test/fixtures/profileDetails";
import { previewChanges, previewNone, savedChanges, savedNew } from "../../test/fixtures/saves";
import { demoConnected, openRgbConnected } from "../../test/fixtures/sessions";
import { viewArmedRetrying, viewConnectApplied, viewFiredRetrying, viewStoppedArmed } from "../../test/fixtures/views";
import type { ConnectOutcome } from "../../lib/api/bindings/ConnectOutcome";
import type { ShellView } from "../../lib/api/bindings/ShellView";
import {
  storeEmpty,
  storeOffline,
  storeRecovered,
  storeSaved,
  storeTwo,
  storeUnsaved,
} from "../../test/fixtures/stores";

const EVENING = "p-0000000000000001";
const EXPORT_TEXT =
  "The file holds the profile name, its settings, the model names of its devices and the program names you typed for “Another program”. It leaves out serial numbers, device locations, your names for devices and zones, and the profile's id on this computer.";
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
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Profiles" }));
  return screen.findByRole("navigation", { name: "Profiles" });
}

async function chooseProfile(name: string) {
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: new RegExp(`^${name.replace(/[()]/g, "\\$&")} \\d`) }));
  return screen.findByRole("article", { name });
}

async function openSaveDialog() {
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: "Save current lighting…" }));
  return screen.findByRole("dialog", { name: "Save the current lighting" });
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
  expect(result().textContent).toMatch(/until you disconnect, switch between demo mode and OpenRGB, or close PrismForge\./);
  expect(result().textContent).not.toMatch(/until it closes/);
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

test("a_marked_profile_that_waits_for_a_connection_says_what_the_trigger_will_do", async () => {
  const shell = fakeShell(demoConnected, handlers(storeTwo, { apply_profile: () => waitingForConnection }));
  render(App);
  const article = await chooseProfile("Evening");
  await shell.pushView({ ...shell.view(demoConnected), revision: 50, on_connect: { Fired: { ran: viewConnectApplied.connect_apply, next: { Ready: { name: "Evening" } } } } });

  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening was not applied now: PrismForge is not connected\./));
  expect(result().textContent).toMatch(/PrismForge applied Evening when it connected\. All 4 settings were accepted\. Apply on connect does not run again by itself in this session/);
  await shell.pushView({ ...shell.view(demoConnected), revision: 60, on_connect: { Armed: { id: EVENING, name: "Evening" } } });
  await waitFor(() => expect(result().textContent).toMatch(/^\s*Evening will be applied as soon as PrismForge has read the devices\./));
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

  await fireEvent.click(within(article).getByRole("button", { name: "Export…" }));
  const dialog = await screen.findByRole("dialog", { name: "Export Evening" });
  expect(within(dialog).getByText(EXPORT_TEXT)).toBeTruthy();
  expect(dialog.textContent).not.toMatch(/identify this computer/);
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
    "When PrismForge connects, it applies this profile once. PrismForge connects by itself only when “Connect to OpenRGB when PrismForge starts” is on in Settings.",
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

test("the_status_strip_shows_the_active_profile_and_unknown_unsaved_changes", async () => {
  const unknown = structuredClone(storeSaved);
  unknown.summary.unsaved_changes = null;
  let store: StoreView = storeUnsaved;
  fakeShell(demoConnected, handlers(storeUnsaved, { profile_store: () => store }));
  render(App);

  const status = await screen.findByRole("button", { name: /^Profile: Evening/ });
  expect(status.textContent).toMatch(/4 unsaved changes/);
  store = unknown;
  const panel = await openProfileChip();
  await fireEvent.click(within(panel).getByRole("button", { name: "Open Profiles" }));
  await waitFor(() => expect(screen.getByRole("button", { name: /^Profile: Evening/ }).textContent).toMatch(/Unsaved changes unknown/));
  expect(screen.getByRole("navigation", { name: "Profiles" })).toBeTruthy();
});

test("busy_is_shown_and_not_retried", async () => {
  const shell = fakeShell(demoConnected, handlers(storeEmpty, { create_profile: refuse(errors.sessionBusy) }));
  render(App);
  await openSaveDialog();

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
  await openSaveDialog();
  expect(screen.getByText(/The new profile holds the effects, hardware modes, program statements and frame rate in use now\./)).toBeTruthy();

  await fireEvent.input(screen.getByRole("textbox", { name: "Profile name" }), { target: { value: "  Evening " } });
  await fireEvent.click(screen.getByRole("button", { name: "Save as new profile" }));

  expect(called(shell, "create_profile").map((call) => call.args)).toEqual([{ name: "Evening" }]);
  await waitFor(() => expect(result().textContent).toMatch(/Saved the current lighting as a new profile on this computer\./));
});

test("save_as_new_with_an_active_profile_says_what_it_holds", async () => {
  fakeShell(demoConnected, handlers(storeUnsaved));
  render(App);
  await openSaveDialog();

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
  expect(within(article).getByText(/until you disconnect, switch between demo mode and OpenRGB, or close PrismForge\./)).toBeTruthy();
  expect(within(article).queryByText(/until PrismForge closes/)).toBeNull();
});

test("demo_profiles_are_marked_as_demo_on_the_profiles_screen", async () => {
  fakeShell(demoConnected, handlers(storeSaved));
  render(App);
  await openProfiles();

  expect(await findDemoMarker()).toBeTruthy();
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

function withRate(refused: boolean, items: boolean): typeof appliedFully {
  const outcome = structuredClone(appliedFully);
  if (typeof outcome === "object") {
    if (refused) {
      outcome.Applied.frame_rate = { Refused: { error: { Session: "Busy" } } };
    }
    if (!items) {
      outcome.Applied.items = [];
    }
  }
  return outcome;
}

test("a_refused_frame_rate_makes_the_apply_partial", async () => {
  fakeShell(demoConnected, handlers(storeTwo, { apply_profile: () => withRate(true, true) }));
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening is applied in part\. The frame rate was refused\./));
  expect(result().textContent).not.toMatch(/Evening is applied\. /);
});

test("a_profile_without_settings_says_it_had_nothing_to_apply", async () => {
  fakeShell(demoConnected, handlers(storeTwo, { apply_profile: () => withRate(false, false) }));
  render(App);
  const article = await chooseProfile("Evening");

  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await waitFor(() => expect(result().textContent).toMatch(/Evening has nothing to apply: it holds no effect, hardware mode or program statement\./));
  expect(result().textContent).not.toMatch(/All 0 settings/);
});

test("results_from_the_demo_store_go_when_a_session_with_openrgb_takes_over", async () => {
  const real = structuredClone(storeOffline);
  let store: StoreView = storeTwo;
  const shell = fakeShell(
    demoConnected,
    handlers(storeTwo, {
      profile_store: () => store,
      apply_profile: () => appliedFully,
      import_profile: () => importedRenamed,
    }),
  );
  render(App);
  const article = await chooseProfile("Evening");
  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));
  await waitFor(() => expect(result().textContent).toMatch(/Evening is applied\./));
  await fireEvent.click(screen.getByRole("button", { name: "Import a profile" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "Import result" }).textContent).toMatch(/Imported as/));
  await fireEvent.click(within(await screen.findByRole("article", { name: "Evening (2)" })).getByRole("button", { name: "Delete" }));
  expect(screen.getByRole("dialog")).toBeTruthy();

  store = real;
  await shell.push(openRgbConnected);

  await waitFor(() => expect(result().textContent?.trim()).toBe(""));
  expect(screen.getByRole("status", { name: "Import result" }).textContent?.trim()).toBe("");
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("results_go_after_disconnect_and_reconnect", async () => {
  const shell = fakeShell(
    demoConnected,
    handlers(storeTwo, {
      apply_profile: () => appliedFully,
      disconnect: () => ({ revision: 100, session: null, applying: null, connect_apply: null, on_connect: "NotApplicable" }),
    }),
  );
  render(App);
  const article = await chooseProfile("Evening");
  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));
  await waitFor(() => expect(result().textContent).toMatch(/Evening is applied\./));
  await fireEvent.click(within(article).getByRole("button", { name: "Rename" }));
  expect(screen.getByRole("textbox", { name: "New name" })).toBeTruthy();

  await connectionAction("Disconnect");
  await shell.pushView({ revision: 200, session: demoConnected, applying: null, connect_apply: null, on_connect: "NotApplicable" });

  await waitFor(() => expect(screen.getByRole("navigation", { name: "Main" }).textContent).toMatch(/Devices/));
  expect(result().textContent?.trim()).toBe("");
  expect(screen.queryByRole("textbox", { name: "New name" })).toBeNull();
});

async function deskWith(view: ShellView) {
  const session = view.session;
  const shell = fakeShell(session, { profile_store: () => storeOffline, profile_details: () => detailsOffline });
  render(App);
  await shell.pushView({ ...view, revision: 50 });
  const list = await openProfiles();
  await fireEvent.click(within(list).getByRole("button", { name: /^Desk/ }));
  return screen.findByRole("article", { name: "Desk" });
}

function onConnect() {
  return screen.getByRole("status", { name: "Apply on connect" });
}

test("an_armed_profile_is_said_to_be_applied_when_the_devices_are_read_even_while_retrying", async () => {
  const article = await deskWith(viewArmedRetrying);

  await waitFor(() => expect(onConnect().textContent?.trim()).toBe("PrismForge will apply Desk as soon as it has read the devices."));
  expect(within(article).getByText(/Apply is off\. PrismForge is not connected right now\. PrismForge will apply Desk as soon as it has read the devices\./)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/not applied again/);
});

test("a_fired_profile_is_said_not_to_apply_again_while_the_session_retries", async () => {
  await deskWith(viewFiredRetrying);

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "PrismForge applied Desk when it connected. Its setting was accepted. Apply on connect does not run again by itself in this session, also not when the connection comes back. You can still apply a profile yourself. After you disconnect and connect, PrismForge applies Desk by itself.",
    ),
  );
});

test("a_session_that_stopped_by_itself_says_to_connect_again", async () => {
  const article = await deskWith(viewStoppedArmed);

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "The connection stopped and PrismForge does not try again. Connect again on the Devices screen. When you connect, PrismForge applies Desk as soon as it has read the devices.",
    ),
  );
  expect(within(article).queryByText(/trying again|not connected yet|once the connection is made/)).toBeNull();
});

test("without_a_marked_profile_nothing_is_promised", async () => {
  await deskWith({ ...viewArmedRetrying, on_connect: "NotArmed" });

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "No profile is marked to apply on connect. A profile you mark before PrismForge has read the devices is applied then.",
    ),
  );
});

test("a_result_that_arrives_after_the_session_ended_is_dropped", async () => {
  let finish: (value: unknown) => void = () => undefined;
  const shell = fakeShell(
    demoConnected,
    handlers(storeTwo, { apply_profile: () => new Promise((resolve) => (finish = resolve)) }),
  );
  render(App);
  const article = await chooseProfile("Evening");
  await fireEvent.click(within(article).getByRole("button", { name: "Apply profile" }));

  await shell.push(openRgbConnected);
  await wait(50);
  finish(appliedFully);
  await wait(250);

  expect(result().textContent?.trim()).toBe("");
});

function firedWith(outcome: ConnectOutcome, name: string | null = "Desk"): ShellView {
  const view = structuredClone(viewFiredRetrying);
  const state = view.on_connect;
  if (typeof state !== "object" || !("Fired" in state) || !state.Fired.ran) {
    throw new Error("fixture is not fired with a result");
  }
  state.Fired.ran = { ...state.Fired.ran, name, outcome };
  view.connect_apply = state.Fired.ran;
  return view;
}

function deskReport(change: (report: Extract<ConnectOutcome, { Applied: unknown }>["Applied"]["report"]) => void): ConnectOutcome {
  const state = viewFiredRetrying.on_connect;
  if (typeof state !== "object" || !("Fired" in state) || !state.Fired.ran || !("Applied" in state.Fired.ran.outcome)) {
    throw new Error("fixture is not an applied result");
  }
  const report = structuredClone(state.Fired.ran.outcome.Applied.report);
  change(report);
  return { Applied: { report } };
}

async function statusAndNotice(view: ShellView) {
  await deskWith(view);
  await waitFor(() => expect(screen.getByRole("region", { name: "Notices" })).toBeTruthy());
  await expandNotices();
  const notice = await screen.findByRole("status", { name: "Applied on connect" });
  await waitFor(() => expect(onConnect().textContent).toMatch(/does not run again by itself/));
  return { status: onConnect().textContent?.trim() ?? "", notice: notice.textContent ?? "" };
}

test("a_fired_apply_with_a_refused_frame_rate_reads_as_partial_in_the_status_and_the_notice", async () => {
  const { status, notice } = await statusAndNotice(
    firedWith(deskReport((report) => (report.frame_rate = { Refused: { error: { Session: "Busy" } } }))),
  );

  const headline = "PrismForge applied Desk in part when it connected. The frame rate was refused.";
  expect(status.startsWith(headline)).toBe(true);
  expect(notice).toContain(headline);
  expect(status).not.toMatch(/Its setting was accepted/);
});

test("a_fired_apply_where_every_setting_was_skipped_reads_the_same_in_the_status_and_the_notice", async () => {
  const { status, notice } = await statusAndNotice(
    firedWith(
      deskReport((report) => {
        for (const item of report.items) {
          item.outcome = { Skipped: { reason: "Missing" } };
        }
      }),
    ),
  );

  const headline = "PrismForge connected. Nothing from Desk was applied. Its setting was skipped and stays in the profile.";
  expect(status.startsWith(headline)).toBe(true);
  expect(notice).toContain(headline);
});

test("a_failed_apply_on_connect_reads_the_same_in_the_status_and_the_notice", async () => {
  const { status, notice } = await statusAndNotice(
    firedWith(
      { Failed: { error: { Store: { NewerFile: { file: "profiles/p-0000000000000003.json", problem: { NewerVersion: { found: 2, supported: 1 } } } } } } },
      null,
    ),
  );

  const headline = "PrismForge could not apply the profile marked to apply on connect when it connected.";
  expect(status.startsWith(headline)).toBe(true);
  expect(notice).toContain(headline);
  expect(status).toMatch(/needs a newer PrismForge/);
});

test("an_unreadable_marked_profile_says_nothing_is_applied_and_why", async () => {
  await deskWith({ ...viewArmedRetrying, on_connect: { Unreadable: { id: "p-0000000000000003", problem: "NewerVersion" } } });

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "The profile marked to apply on connect needs a newer PrismForge, so this version cannot read it. Nothing is applied when PrismForge connects.",
    ),
  );
});

test("a_missing_marked_profile_says_nothing_is_applied_and_why", async () => {
  await deskWith({ ...viewArmedRetrying, on_connect: { Unreadable: { id: "p-0000000000000003", problem: "Missing" } } });

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "The profile marked to apply on connect is not in the store any more. Nothing is applied when PrismForge connects.",
    ),
  );
});

test("after_it_fired_with_nothing_marked_a_new_mark_is_not_applied_by_itself", async () => {
  await deskWith({ ...viewFiredRetrying, connect_apply: null, on_connect: { Fired: { ran: null, next: { Ready: { name: "Desk" } } } } });

  await waitFor(() =>
    expect(onConnect().textContent?.trim()).toBe(
      "No profile was marked when PrismForge connected, so none was applied by itself. A profile you mark now is not applied by itself in this session. You can still apply it yourself. After you disconnect and connect, PrismForge applies Desk by itself.",
    ),
  );
  expect(onConnect().textContent).not.toMatch(/is not applied in this session/);
});
