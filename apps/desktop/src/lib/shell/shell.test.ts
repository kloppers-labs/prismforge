import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import type { SessionSnapshot } from "../api/bindings/SessionSnapshot";
import type { ShellView } from "../api/bindings/ShellView";
import type { StoreView } from "../api/bindings/StoreView";
import { fakeShell, refuse } from "../../test/backend";
import { effectHandlers, wait } from "../../test/effects";
import { lifecycleTrayStopped } from "../../test/fixtures/lifecycles";
import { detailsEvening } from "../../test/fixtures/profileDetails";
import { previewChanges } from "../../test/fixtures/saves";
import {
  demoConnected,
  demoEffectsNotConnected,
  openRgbConnected,
  openRgbConnecting,
  openRgbDegraded,
  openRgbLostWhileConnected,
  openRgbOldServer,
  openRgbRefused,
} from "../../test/fixtures/sessions";
import { storeEmpty, storeSaved, storeUnsaved } from "../../test/fixtures/stores";
import { viewApplying, viewArmedRetrying, viewConnectApplied, viewStoppedArmed } from "../../test/fixtures/views";
import {
  connectionChip,
  connectionAction,
  current,
  findRail,
  findStrip,
  go,
  notices,
  openConnection,
  openProfileChip,
  profileChip,
  railItem,
  railItems,
  strip,
  waitForConnection,
} from "../../test/shell";

const AREAS = ["Devices", "Effects", "Profiles", "Diagnostics", "Settings"];

afterEach(() => vi.restoreAllMocks());

function shellView(session: SessionSnapshot | null, revision: number, extra: Partial<ShellView> = {}): ShellView {
  return { revision, session, applying: null, connect_apply: null, on_connect: "NotApplicable", ...extra };
}

function withStore(store: StoreView) {
  return { ...effectHandlers(), profile_store: () => store };
}

test("the_rail_shows_the_same_items_in_the_same_order_in_every_state", async () => {
  const shell = fakeShell(null, effectHandlers());
  render(App);
  await findRail();
  const seen = [railItems()];

  for (const [index, session] of [openRgbConnecting, openRgbRefused, openRgbConnected, openRgbLostWhileConnected, demoConnected, openRgbOldServer].entries()) {
    await shell.pushView(shellView(session, 10 + index));
    await waitFor(() => expect(connectionChip().getAttribute("aria-label")).toBeTruthy());
    seen.push(railItems());
  }
  await shell.pushView(shellView(null, 30));
  seen.push(railItems());

  expect(seen.every((items) => JSON.stringify(items) === JSON.stringify(AREAS))).toBe(true);
});

test.each(["Devices", "Effects", "Profiles", "Settings"])("the_current_area_stays_marked_after_a_disconnect_from_%s", async (area) => {
  const shell = fakeShell(demoConnected, { ...withStore(storeEmpty), disconnect: () => shell.view(null) });
  render(App);
  await go(area);
  expect(current()).toEqual([area]);

  await connectionAction("Disconnect");

  await waitFor(() => expect(within(strip()).getByText("Not connected")).toBeTruthy());
  expect(current()).toEqual([area]);
  expect(railItems()).toEqual(AREAS);
});

test("effects_without_a_connection_show_an_empty_state_with_the_way_to_connect", async () => {
  const shell = fakeShell(null, { start_demo: () => shell.view(demoConnected), ...effectHandlers() });
  render(App);
  await go("Effects");

  const main = screen.getByRole("main");
  expect(within(main).getByRole("heading", { level: 1, name: "Effects" })).toBeTruthy();
  expect(main.textContent).toMatch(/Effects need a connection/);
  await fireEvent.click(within(main).getByRole("button", { name: "Connect to OpenRGB" }));

  expect(current()).toEqual(["Devices"]);
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Host" })));
});

test("effects_while_connecting_say_why_and_lead_to_devices", async () => {
  fakeShell(openRgbRefused, effectHandlers());
  render(App);
  await waitForConnection("Retrying, retry 3 next");
  await go("Effects");

  const main = screen.getByRole("main");
  expect(within(main).getByRole("heading", { level: 1, name: "Effects" })).toBeTruthy();
  expect(main.textContent).toMatch(/Nothing answered at the OpenRGB server at 127\.0\.0\.1:6742\./);
  await fireEvent.click(within(main).getByRole("button", { name: "Open Devices" }));
  expect(current()).toEqual(["Devices"]);
});

test("first_run_makes_connecting_obvious", async () => {
  fakeShell(null);
  render(App);

  await screen.findByRole("heading", { level: 1, name: "Connect PrismForge to your lighting" });
  expect(current()).toEqual(["Devices"]);
  expect(screen.getByRole("button", { name: "Connect" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Start demo mode" })).toBeTruthy();
  expect(connectionChip().textContent).toMatch(/Not connected/);
  const panel = await openConnection();
  expect(panel.textContent).toMatch(/PrismForge is not connected\./);
  expect(within(panel).getByRole("button", { name: "Connect to OpenRGB" })).toBeTruthy();
});

test.each([
  ["openRgbConnected", openRgbConnected, "Connected", "OpenRGB 127.0.0.1:6742"],
  ["openRgbConnecting", openRgbConnecting, "Connecting", "OpenRGB 127.0.0.1:6742"],
  ["openRgbRefused", openRgbRefused, "Retrying, retry 3 next", "OpenRGB 127.0.0.1:6742"],
  ["openRgbDegraded", openRgbDegraded, "Connected with problems", "OpenRGB 127.0.0.1:6742"],
  ["openRgbOldServer", openRgbOldServer, "Not connected", "OpenRGB 127.0.0.1:6742"],
  ["demoConnected", demoConnected, "Connected", "Demo backend"],
] as const)("the_strip_shows_the_connection_state_%s", async (_name, session, label, where) => {
  fakeShell(session, effectHandlers());
  render(App);
  await findStrip();
  await waitFor(() => expect(connectionChip().getAttribute("aria-label")).toBe(`Connection: ${label}, ${where}`));

  const chip = connectionChip();
  expect(within(chip).getByText(label)).toBeTruthy();
  expect(within(chip).getByText(where)).toBeTruthy();
  expect(chip.getAttribute("aria-label")).toBe(`Connection: ${label}, ${where}`);
});

test("the_connection_panel_gives_the_reason_in_plain_words", async () => {
  fakeShell(openRgbRefused);
  render(App);
  await waitForConnection("Retrying, retry 3 next");

  const panel = await openConnection();

  expect(panel.textContent).toMatch(/Nothing answered at the OpenRGB server at 127\.0\.0\.1:6742\./);
  expect(within(panel).getByRole("button", { name: "Stop retrying" })).toBeTruthy();
  expect(within(panel).queryByRole("button", { name: "Rescan devices" })).toBeNull();
});

test("the_strip_shows_the_active_profile_with_its_unsaved_changes", async () => {
  fakeShell(demoConnected, withStore(storeUnsaved));
  render(App);

  const chip = await within(await findStrip()).findByRole("button", { name: "Profile: Evening, 4 unsaved changes" });
  expect(chip.textContent).toMatch(/Evening/);
  expect(chip.textContent).toMatch(/4 unsaved changes/);
});

test("the_strip_reads_unknown_unsaved_changes_as_unknown", async () => {
  const unknown = structuredClone(storeSaved);
  unknown.summary.unsaved_changes = null;
  fakeShell(demoConnected, withStore(unknown));
  render(App);

  const chip = await within(await findStrip()).findByRole("button", { name: "Profile: Evening, unsaved changes unknown" });
  expect(chip.textContent).toMatch(/Unsaved changes unknown/);
});

test("the_strip_shows_a_profile_apply_in_progress", async () => {
  const shell = fakeShell(demoConnected, withStore(storeSaved));
  render(App);
  await within(await findStrip()).findByRole("button", { name: /^Profile: Evening/ });

  await shell.pushView({ ...viewApplying, revision: 50 });

  const chip = await within(strip()).findByRole("button", { name: "Profile: applying Evening" });
  expect(chip.textContent).toMatch(/Applying Evening/);
  const panel = await openProfileChip();
  expect(panel.textContent).toMatch(/PrismForge is applying the profile Evening\. Wait until it finishes\./);
});

test("the_strip_shows_the_profile_that_applies_on_connect", async () => {
  const shell = fakeShell(viewArmedRetrying.session, withStore(storeEmpty));
  render(App);
  await findStrip();

  await shell.pushView({ ...viewArmedRetrying, revision: 60 });

  const chip = await within(strip()).findByRole("button", { name: "Profile: none active, Desk applies on connect" });
  expect(chip.textContent).toMatch(/Desk applies on connect/);
});

test("demo_mode_is_marked_in_the_strip_on_every_area", async () => {
  fakeShell(demoConnected, withStore(storeEmpty));
  render(App);
  await findStrip();

  for (const area of ["Devices", "Effects", "Profiles", "Settings"]) {
    await go(area);
    const marker = within(strip()).getByText("Demo mode");
    expect(marker.closest("[data-demo]")?.textContent).toMatch(/simulated/);
  }
  await go("Diagnostics");
  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();
  expect(within(strip()).getByText("Demo mode")).toBeTruthy();
});

test("real_hardware_never_shows_the_demo_marker", async () => {
  fakeShell(openRgbConnected, effectHandlers());
  render(App);
  await findStrip();

  expect(within(strip()).queryByText("Demo mode")).toBeNull();
  expect(strip().querySelector("[data-demo]")).toBeNull();
});

test("the_connection_chip_is_keyboard_operable", async () => {
  const shell = fakeShell(openRgbConnected, { ...effectHandlers(), refresh: () => null });
  render(App);
  await waitForConnection("Connected");
  const chip = connectionChip();
  chip.focus();

  await fireEvent.keyDown(chip, { key: "ArrowDown" });
  const panel = screen.getByRole("group", { name: "Connection details" });
  expect(chip.getAttribute("aria-expanded")).toBe("true");
  const rescan = within(panel).getByRole("button", { name: "Rescan devices" });
  expect(document.activeElement).toBe(rescan);

  await fireEvent.keyDown(rescan, { key: "ArrowDown" });
  expect(document.activeElement).toBe(within(panel).getByRole("button", { name: "Disconnect" }));
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "ArrowUp" });
  expect(document.activeElement).toBe(rescan);

  await fireEvent.keyDown(rescan, { key: "Escape" });
  expect(screen.queryByRole("group", { name: "Connection details" })).toBeNull();
  expect(document.activeElement).toBe(chip);
  expect(chip.getAttribute("aria-expanded")).toBe("false");

  await fireEvent.click(chip);
  await fireEvent.click(within(screen.getByRole("group", { name: "Connection details" })).getByRole("button", { name: "Rescan devices" }));
  expect(shell.commands().filter((command) => command === "refresh")).toHaveLength(1);
  expect(document.activeElement).toBe(chip);
});

test("a_click_outside_closes_a_panel", async () => {
  fakeShell(openRgbConnected, effectHandlers());
  render(App);
  await waitForConnection("Connected");
  await openConnection();

  await fireEvent.pointerDown(screen.getByRole("main"));

  expect(screen.queryByRole("group", { name: "Connection details" })).toBeNull();
});

test("stop_retrying_sits_behind_the_connection_chip_and_disconnects", async () => {
  const shell = fakeShell(openRgbRefused, { disconnect: () => shell.view(null) });
  render(App);
  await waitForConnection("Retrying, retry 3 next");

  await connectionAction("Stop retrying");

  expect(shell.commands()).toContain("disconnect");
});

test("the_profile_chip_saves_changes_and_opens_profiles", async () => {
  const shell = fakeShell(demoConnected, {
    ...withStore(storeUnsaved),
    profile_details: () => detailsEvening,
    preview_profile_update: () => previewChanges,
  });
  render(App);
  await within(await findStrip()).findByRole("button", { name: /^Profile: Evening/ });

  const panel = await openProfileChip();
  await fireEvent.click(within(panel).getByRole("button", { name: "Save changes…" }));

  await waitFor(() => expect(shell.calls.find((call) => call.command === "preview_profile_update")?.args).toEqual({ id: "p-0000000000000001" }));
  expect(current()).toEqual(["Profiles"]);

  await go("Devices");
  const again = await openProfileChip();
  await fireEvent.click(within(again).getByRole("button", { name: "Open Profiles" }));
  expect(current()).toEqual(["Profiles"]);
});

test("one_notice_shows_in_the_slot_and_can_be_dismissed", async () => {
  fakeShell(openRgbConnected, { ...effectHandlers(), refresh: refuse({ Session: "Busy" }) });
  render(App);
  await waitForConnection("Connected");

  await connectionAction("Rescan devices");

  const slot = await waitFor(() => notices() ?? Promise.reject(new Error("no notices")));
  const alert = within(slot).getByRole("alert");
  expect(alert.textContent).toMatch(/still working on earlier requests/);
  await fireEvent.click(within(alert).getByRole("button", { name: "Dismiss" }));
  expect(notices()).toBeNull();
});

test("busy_is_shown_and_not_retried", async () => {
  const shell = fakeShell(openRgbConnected, { ...effectHandlers(), refresh: refuse({ Session: "Busy" }) });
  render(App);
  await waitForConnection("Connected");

  await connectionAction("Rescan devices");
  await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/still working on earlier requests/));
  await wait(400);

  expect(shell.commands().filter((command) => command === "refresh")).toHaveLength(1);
});

test("several_notices_collapse_with_a_count_and_each_can_be_dismissed", async () => {
  const shell = fakeShell(demoConnected, {
    ...effectHandlers(),
    refresh: refuse({ Session: "Busy" }),
    lifecycle: () => lifecycleTrayStopped,
  });
  render(App);
  await waitForConnection("Connected");
  await shell.pushView({ ...viewConnectApplied, revision: 50 });
  await connectionAction("Rescan devices");

  const slot = await waitFor(() => notices() ?? Promise.reject(new Error("no notices")));
  await waitFor(() => expect(within(slot).getByRole("button", { name: "Show all 3 notices" })).toBeTruthy());
  expect(within(slot).getByRole("alert").textContent).toMatch(/still working on earlier requests/);
  expect(within(slot).queryByRole("status", { name: "Applied on connect" })).toBeNull();

  await fireEvent.click(within(slot).getByRole("button", { name: "Show all 3 notices" }));
  const applied = within(slot).getByRole("status", { name: "Applied on connect" });
  expect(within(slot).getByRole("status", { name: /^Stop all PrismForge effects, from the tray/ })).toBeTruthy();

  await fireEvent.click(within(applied).getByRole("button", { name: "Dismiss" }));
  await fireEvent.click(within(within(slot).getByRole("alert")).getByRole("button", { name: "Dismiss" }));
  expect(within(slot).queryByRole("button", { name: /notices/ })).toBeNull();
  expect(within(slot).getByRole("status", { name: /^Stop all PrismForge effects, from the tray/ })).toBeTruthy();
});

test("a_connection_problem_stays_in_the_slot_without_a_dismiss", async () => {
  fakeShell(openRgbLostWhileConnected);
  render(App);

  const notice = await screen.findByRole("status", { name: "Connection status" });
  expect(notices()?.contains(notice)).toBe(true);
  expect(within(notice).queryByRole("button", { name: "Dismiss" })).toBeNull();
});

test("area_shortcuts_move_between_areas", async () => {
  fakeShell(demoConnected, withStore(storeEmpty));
  render(App);
  await findRail();

  await fireEvent.keyDown(document.body, { key: "2", ctrlKey: true });
  expect(current()).toEqual(["Effects"]);
  expect(document.activeElement).toBe(screen.getByRole("main"));
  await fireEvent.keyDown(document.body, { key: "3", ctrlKey: true });
  expect(current()).toEqual(["Profiles"]);
  await fireEvent.keyDown(document.body, { key: ",", ctrlKey: true });
  expect(current()).toEqual(["Settings"]);
  await fireEvent.keyDown(document.body, { key: "1", ctrlKey: true });
  expect(current()).toEqual(["Devices"]);
  expect(railItem("Effects").getAttribute("aria-keyshortcuts")).toBe("Control+2");
});

test("area_shortcuts_do_not_fire_while_typing", async () => {
  fakeShell(null);
  render(App);
  const host = await screen.findByRole("textbox", { name: "Host" });
  host.focus();

  await fireEvent.keyDown(host, { key: "2", ctrlKey: true });

  expect(current()).toEqual(["Devices"]);
});

test("f6_moves_between_the_rail_the_strip_and_the_page", async () => {
  fakeShell(demoConnected, withStore(storeEmpty));
  render(App);
  await findStrip();

  await fireEvent.keyDown(document.body, { key: "F6" });
  expect(document.activeElement).toBe(railItem("Devices"));
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "F6" });
  expect(document.activeElement).toBe(connectionChip());
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "F6" });
  expect(document.activeElement).toBe(screen.getByRole("main"));
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "F6", shiftKey: true });
  expect(document.activeElement).toBe(connectionChip());
});

test("the_first_tab_stop_skips_to_the_content", async () => {
  fakeShell(demoConnected);
  render(App);
  await findStrip();

  const skip = screen.getByRole("button", { name: "Skip to content" });
  expect(document.body.querySelector("button")).toBe(skip);
  await fireEvent.click(skip);
  expect(document.activeElement).toBe(screen.getByRole("main"));
});

test("opening_diagnostics_keeps_the_focused_page_control_in_view", async () => {
  fakeShell(demoConnected, withStore(storeEmpty));
  render(App);
  const list = await screen.findByRole("navigation", { name: /Devices/ });
  const device = within(list).getByRole("button", { name: /^Demo Graphics Card/ });
  device.focus();
  Element.prototype.scrollIntoView ??= () => {};
  const reveal = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});

  await go("Diagnostics");

  await waitFor(() => expect(reveal.mock.contexts).toContain(device));
  expect(reveal.mock.calls.find((_call, index) => reveal.mock.contexts[index] === device)?.[0]).toEqual({ block: "nearest", inline: "nearest" });
  expect(railItem("Diagnostics").getAttribute("aria-expanded")).toBe("true");
  expect(screen.getByRole("main").contains(screen.getByRole("complementary", { name: "Diagnostics" }))).toBe(false);
});

test("escape_in_a_panel_does_not_close_the_diagnostics_drawer", async () => {
  fakeShell(openRgbConnected, effectHandlers());
  render(App);
  await go("Diagnostics");
  const panel = await openConnection();

  await fireEvent.keyDown(within(panel).getAllByRole("button")[0] as HTMLElement, { key: "Escape" });

  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();
  expect(screen.queryByRole("group", { name: "Connection details" })).toBeNull();
});

test("a_stopped_session_shows_the_connect_way_in_effects", async () => {
  fakeShell(viewStoppedArmed.session, effectHandlers());
  render(App);
  await go("Effects");

  expect(within(screen.getByRole("main")).getByRole("button", { name: "Connect to OpenRGB" })).toBeTruthy();
});

test("effects_keep_their_paused_screen_when_the_demo_connection_drops", async () => {
  fakeShell(demoEffectsNotConnected, effectHandlers());
  render(App);
  await go("Effects");

  expect(await screen.findByRole("navigation", { name: "Targets" })).toBeTruthy();
  expect(within(strip()).getByText("Demo mode")).toBeTruthy();
});

test("below_1100_px_diagnostics_covers_an_inert_page_until_an_area_is_chosen", async () => {
  vi.stubGlobal("matchMedia", (query: string) => ({ matches: true, media: query, addEventListener() {}, removeEventListener() {} }));
  fakeShell(demoConnected, withStore(storeEmpty));
  render(App);
  await findStrip();

  await go("Diagnostics");

  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();
  expect(screen.getByRole("main").inert).toBe(true);
  expect(within(strip()).getByText("Demo mode")).toBeTruthy();
  await fireEvent.keyDown(document.body, { key: "2", ctrlKey: true });
  expect(screen.queryByRole("complementary", { name: "Diagnostics" })).toBeNull();
  expect(current()).toEqual(["Effects"]);
  expect(screen.getByRole("main").inert).toBe(false);
  vi.unstubAllGlobals();
});
