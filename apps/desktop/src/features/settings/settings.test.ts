import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { LifecycleView } from "../../lib/api/bindings/LifecycleView";
import { fakeShell, refuse } from "../../test/backend";
import * as errors from "../../test/fixtures/errors";
import {
  lifecycleAsking,
  lifecycleChosen,
  lifecycleEntryUnknown,
  lifecycleFresh,
  lifecycleInUse,
  lifecycleMachineEntry,
  lifecycleQuitting,
  lifecycleNoTray,
  lifecycleRecovered,
  lifecycleStartPath,
  lifecycleTrayStopBusy,
  lifecycleTrayStopped,
} from "../../test/fixtures/lifecycles";
import { demoConnected } from "../../test/fixtures/sessions";

const AUTOMATIC =
  "Connecting alone writes nothing to your devices. Without a click, PrismForge changes your lighting only through a rule you switched on in Automation: when PrismForge starts or connects, when the system sleeps, wakes or ends your session, or when PrismForge quits. At system start that needs “Connect to OpenRGB when PrismForge starts” and a rule that applies a profile when PrismForge starts or connects.";

function at(view: LifecycleView, revision: number): LifecycleView {
  return { ...view, revision };
}

function called(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command);
}

async function openSettings() {
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Settings" }));
  return screen.findByRole("heading", { level: 1, name: "Settings" });
}

function toggle(name: string) {
  return screen.getByRole("switch", { name: new RegExp(`^${name}`) });
}

function radio(name: string) {
  return screen.getByRole("radio", { name });
}

test("settings_show_the_real_state_from_rust", async () => {
  fakeShell(null, { lifecycle: () => lifecycleChosen });
  render(App);
  await openSettings();

  await waitFor(() => expect(radio("Keep PrismForge running in the system tray").getAttribute("aria-checked")).toBe("true"));
  expect(radio("Quit PrismForge").getAttribute("aria-checked")).toBe("false");
  expect(toggle("Start PrismForge when you sign in").getAttribute("aria-checked")).toBe("true");
  expect(toggle("Start hidden in the system tray").getAttribute("aria-checked")).toBe("true");
  expect(toggle("Connect to OpenRGB when PrismForge starts").getAttribute("aria-checked")).toBe("true");
  expect(screen.getByText(/PrismForge connects to/).textContent).toBe(
    "PrismForge connects to 192.168.1.20:6742, the address of your last Connect. If OpenRGB does not answer, it keeps retrying, as after Connect. It never starts demo mode.",
  );
  expect(screen.getByText("store-folder")).toBeTruthy();
});

test("every_setting_is_off_and_nothing_is_chosen_at_first", async () => {
  fakeShell(null);
  render(App);
  await openSettings();

  await waitFor(() => expect(screen.getByText("Not chosen yet. PrismForge asks the first time you close the window.")).toBeTruthy());
  expect(radio("Keep PrismForge running in the system tray").getAttribute("aria-checked")).toBe("false");
  expect(radio("Quit PrismForge").getAttribute("aria-checked")).toBe("false");
  for (const name of ["Start PrismForge when you sign in", "Start hidden in the system tray", "Connect to OpenRGB when PrismForge starts"]) {
    expect(toggle(name).getAttribute("aria-checked")).toBe("false");
  }
  const connect = toggle("Connect to OpenRGB when PrismForge starts") as HTMLButtonElement;
  expect(connect.disabled).toBe(true);
  expect(document.getElementById(connect.getAttribute("aria-describedby") ?? "")?.textContent?.trim()).toBe(
    "Connect to OpenRGB once first. PrismForge then remembers that address.",
  );
});

test("a_switch_sends_its_change_and_shows_the_state_rust_returns", async () => {
  const shell = fakeShell(null, { change_setting: () => at(lifecycleChosen, 20) });
  render(App);
  await openSettings();

  await fireEvent.click(await waitFor(() => toggle("Start PrismForge when you sign in")));

  expect(called(shell, "change_setting").map((call) => call.args)).toEqual([{ change: { StartWithSystem: true } }]);
  await waitFor(() => expect(toggle("Start PrismForge when you sign in").getAttribute("aria-checked")).toBe("true"));
});

test("the_close_choice_is_sent_from_the_radio_group", async () => {
  const shell = fakeShell(null, { change_setting: () => at(lifecycleChosen, 20) });
  render(App);
  await openSettings();

  await fireEvent.click(await waitFor(() => radio("Keep PrismForge running in the system tray")));

  expect(called(shell, "change_setting").map((call) => call.args)).toEqual([{ change: { OnClose: "Background" } }]);
});

test("a_refused_start_entry_is_shown_once_and_the_switch_stays_off", async () => {
  const shell = fakeShell(null, { change_setting: refuse(errors.settingStartEntryRefused) });
  render(App);
  await openSettings();

  await fireEvent.click(await waitFor(() => toggle("Start PrismForge when you sign in")));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toBe(
    "PrismForge could not add PrismForge to the programs that start with the system: Access is denied.. The switch shows what the system reports now.",
  );
  expect(toggle("Start PrismForge when you sign in").getAttribute("aria-checked")).toBe("false");
  expect(called(shell, "change_setting")).toHaveLength(1);
});

test("a_start_entry_change_the_system_did_not_keep_says_so", async () => {
  fakeShell(null, { change_setting: refuse(errors.settingStartEntryNotKept) });
  render(App);
  await openSettings();

  await fireEvent.click(await waitFor(() => toggle("Start PrismForge when you sign in")));

  expect((await screen.findByRole("alert")).textContent).toBe(
    "PrismForge tried to add PrismForge to the programs that start with the system, but the system did not keep the change. The switch shows what the system reports now.",
  );
});

test("an_unreadable_start_entry_is_shown_as_unknown", async () => {
  fakeShell(null, { lifecycle: () => lifecycleEntryUnknown });
  render(App);
  await openSettings();

  expect(await screen.findByText("PrismForge cannot read its start entry: No home folder was found.")).toBeTruthy();
  expect(toggle("Start PrismForge when you sign in").getAttribute("aria-checked")).toBe("false");
});

test("the_text_about_automatic_lighting_changes_is_exact", async () => {
  fakeShell(null);
  render(App);
  await openSettings();

  const note = (await screen.findByText("When your lighting changes without a click")).closest("details") as HTMLElement;
  const lines = within(note).getAllByRole("paragraph").map((line) => line.textContent?.trim());
  expect(lines).toEqual([AUTOMATIC, "PrismForge effects run only while PrismForge runs and is connected. Nothing is stored on your devices."]);
});

test("without_a_tray_background_running_and_start_hidden_are_unavailable_and_say_why", async () => {
  fakeShell(null, { lifecycle: () => lifecycleNoTray });
  render(App);
  await openSettings();

  await waitFor(() => expect((radio("Keep PrismForge running in the system tray") as HTMLButtonElement).disabled).toBe(true));
  expect(
    screen.getByText(
      "Running in the background is not available: this system has no tray (The system has no tray area.). Closing the window quits PrismForge, and its effects stop.",
    ),
  ).toBeTruthy();
  expect((toggle("Start hidden in the system tray") as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText("Not available: this system has no tray, so the window always opens.")).toBeTruthy();
  expect(screen.queryByText(/Not chosen yet/)).toBeNull();
});

test("settings_held_by_another_prismforge_say_so", async () => {
  fakeShell(null, { lifecycle: () => lifecycleInUse });
  render(App);
  await openSettings();

  const alert = await screen.findByRole("alert");
  expect(alert.textContent?.trim()).toMatch(/^Another PrismForge window is using the profile folder\./);
  expect(alert.textContent?.trim()).toMatch(/The settings below cannot be changed now, and every one of them counts as off\.$/);
  expect((radio("Quit PrismForge") as HTMLButtonElement).disabled).toBe(true);
});

test("a_repaired_settings_file_is_explained", async () => {
  fakeShell(null, { lifecycle: () => lifecycleRecovered });
  render(App);
  await openSettings();

  expect(await screen.findByText(/^PrismForge could not read app-settings\.json and used its backup\./)).toBeTruthy();
  expect(screen.getByText("Your last settings change may be missing. Check the settings on this screen.")).toBeTruthy();
  expect(screen.getByText("The damaged file is still there, now called app-settings.json.unreadable-1.")).toBeTruthy();
});

test("the_first_close_explains_background_running_and_sends_the_choice", async () => {
  const shell = fakeShell(demoConnected, { answer_close: () => null });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });

  await shell.pushLifecycle(at(lifecycleAsking, 30));

  const dialog = await screen.findByRole("dialog", { name: "Keep PrismForge running after you close the window?" });
  const text = dialog.textContent ?? "";
  expect(text).toContain("the window closes and PrismForge stays in the system tray. Your PrismForge effects keep running on your devices.");
  expect(text).toContain("To quit completely, choose Quit PrismForge in that menu, on the Settings screen, or press Ctrl+Q.");
  expect(text).toContain("PrismForge remembers your choice. You can change it on the Settings screen.");
  expect(document.activeElement?.textContent).toBe("Cancel");
  await fireEvent.click(within(dialog).getByRole("button", { name: "Keep running in the system tray" }));

  expect(called(shell, "answer_close").map((call) => call.args)).toEqual([{ choice: "Background" }]);
});

test("escape_cancels_the_first_close_and_quit_is_offered", async () => {
  const shell = fakeShell(null, { answer_close: () => null });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });
  await shell.pushLifecycle(at(lifecycleAsking, 30));
  const dialog = await screen.findByRole("dialog");

  expect(within(dialog).getByRole("button", { name: "Quit PrismForge" })).toBeTruthy();
  await fireEvent.keyDown(dialog, { key: "Escape" });

  expect(called(shell, "answer_close").map((call) => call.args)).toEqual([{ choice: null }]);
  await shell.pushLifecycle(at(lifecycleFresh, 31));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

test("quit_is_on_the_settings_screen_and_on_the_keyboard", async () => {
  const shell = fakeShell(null, { quit: () => null });
  render(App);
  await openSettings();

  await fireEvent.click(screen.getByRole("button", { name: "Quit PrismForge" }));
  await waitFor(() => expect(called(shell, "quit")).toHaveLength(1));
  expect(screen.getByText(/^Stops PrismForge effects, puts back the lighting from before where it can, disconnects and ends PrismForge\./)).toBeTruthy();
});

test("ctrl_q_quits_where_it_is_the_quit_shortcut", async () => {
  const shell = fakeShell(null, { quit: () => null });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });

  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });

  await waitFor(() => expect(called(shell, "quit")).toHaveLength(1));
});

test("the_tray_stop_report_lists_each_device_when_the_window_is_shown", async () => {
  const shell = fakeShell(demoConnected);
  render(App);
  await screen.findByRole("navigation", { name: "Main" });

  await shell.pushLifecycle(at(lifecycleTrayStopped, 40));

  const report = await screen.findByRole("status", { name: /^Stop all PrismForge effects, from the tray/ });
  expect(within(report).getByText("Stop all PrismForge effects, from the tray: every effect was stopped.")).toBeTruthy();
  await fireEvent.click(within(report).getByRole("button", { name: "Details" }));
  expect(within(report).getByText("Demo Graphics Card")).toBeTruthy();
  expect(within(report).getByText("Demo Motherboard")).toBeTruthy();
  await fireEvent.click(within(report).getByRole("button", { name: "Dismiss" }));
  expect(screen.queryByRole("status", { name: /^Stop all PrismForge effects/ })).toBeNull();
});

test("busy_in_the_tray_stop_report_is_shown_and_not_retried", async () => {
  const shell = fakeShell(demoConnected);
  render(App);
  await screen.findByRole("navigation", { name: "Main" });

  await shell.pushLifecycle(at(lifecycleTrayStopBusy, 40));

  const report = await screen.findByRole("status", { name: /^Stop all PrismForge effects, from the tray/ });
  expect(within(report).getByText("Stop all PrismForge effects, from the tray: some effects were not stopped.")).toBeTruthy();
  expect(
    within(report).getByText("Not stopped. PrismForge is still working on earlier requests. Wait a moment, then try again."),
  ).toBeTruthy();
  expect(within(report).getByText("PrismForge does not try again by itself. Stop the remaining effects on the Effects screen.")).toBeTruthy();
  expect(called(shell, "stop_effect")).toHaveLength(0);
});

test("the_start_up_path_shows_each_step_and_the_profile_it_applies", async () => {
  fakeShell(null, { lifecycle: () => lifecycleStartPath });
  render(App);
  await openSettings();

  const steps = await screen.findByRole("list", { name: "Start-up steps" });
  const items = within(steps).getAllByRole("listitem");
  expect(items).toHaveLength(4);
  expect(within(items[0]!).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(items[1]!).getByRole("switch").getAttribute("aria-checked")).toBe("false");
  expect(within(items[2]!).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(items[3]!).getByText("Evening")).toBeTruthy();
  expect(within(items[3]!).getByText(/by the rule “When PrismForge connects”, once, as soon as the connection is made\./)).toBeTruthy();
  expect(screen.getByText("With steps 1, 3 and 4 on, your lights change at system start without a click. Every step is off until you turn it on.")).toBeTruthy();
  expect(screen.getByRole("status", { name: "Start-up result" }).textContent?.trim()).toBe(
    "When you sign in, PrismForge starts, connects to OpenRGB at 127.0.0.1:6742 and applies Evening. Your lights change at system start without a click.",
  );
});

test("without_a_rule_that_applies_a_profile_the_start_up_path_says_none_and_links_to_automation", async () => {
  const shell = fakeShell(null);
  render(App);
  await openSettings();

  const steps = await screen.findByRole("list", { name: "Start-up steps" });
  const last = within(steps).getAllByRole("listitem")[3]!;
  expect(within(last).getByText("No rule applies a profile when PrismForge starts or connects.")).toBeTruthy();
  expect(screen.getByRole("status", { name: "Start-up result" }).textContent?.trim()).toBe(
    "Your lights do not change by themselves at system start. Not set: start PrismForge when you sign in, connect to OpenRGB when PrismForge starts, a rule that applies a profile when PrismForge starts or connects.",
  );
  await fireEvent.click(within(last).getByRole("button", { name: "Open Automation" }));

  await screen.findByRole("heading", { level: 1, name: "Automation" });
  expect(called(shell, "automation").length).toBeGreaterThan(0);
  expect(screen.queryByRole("heading", { level: 1, name: "Settings" })).toBeNull();
});

test("a_close_answer_that_could_not_be_stored_is_shown_when_the_window_is_back", async () => {
  const shell = fakeShell(demoConnected, { answer_close: refuse(errors.settingsInUse) });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });
  await shell.pushLifecycle(at(lifecycleAsking, 30));
  const dialog = await screen.findByRole("dialog");

  await fireEvent.click(within(dialog).getByRole("button", { name: "Keep running in the system tray" }));
  await shell.pushLifecycle(at(lifecycleFresh, 31));

  const alert = await screen.findByRole("alert");
  expect(
    within(alert).getByText(
      "PrismForge could not remember your choice for closing the window. Another PrismForge window is using the profile folder. This window does not read or save profiles. Close the other window, then try again. It asks again the next time you close the window.",
    ),
  ).toBeTruthy();
});

test("further_quit_requests_are_ignored_while_prismforge_quits", async () => {
  const shell = fakeShell(null, { lifecycle: () => lifecycleQuitting, quit: () => null });
  render(App);
  await openSettings();

  const quit = await screen.findByRole("button", { name: "Quitting PrismForge" });
  expect((quit as HTMLButtonElement).disabled).toBe(true);
  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });

  expect(called(shell, "quit")).toHaveLength(0);
});

test("a_held_quit_shortcut_sends_one_quit", async () => {
  const shell = fakeShell(null, { quit: () => new Promise(() => {}) });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });

  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });
  await fireEvent.keyDown(window, { key: "q", ctrlKey: true, repeat: true });
  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });

  await waitFor(() => expect(called(shell, "quit")).toHaveLength(1));
});

test("the_close_choice_can_be_reached_by_tab_without_a_tray", async () => {
  fakeShell(null, { lifecycle: () => lifecycleNoTray });
  render(App);
  await openSettings();

  await waitFor(() => expect(radio("Quit PrismForge").getAttribute("tabindex")).toBe("0"));
  expect(radio("Keep PrismForge running in the system tray").getAttribute("tabindex")).toBe("-1");
});

test("an_unreadable_start_entry_makes_the_start_up_result_unknown", async () => {
  fakeShell(null, { lifecycle: () => lifecycleEntryUnknown });
  render(App);
  await openSettings();

  expect((await screen.findByRole("status", { name: "Start-up result" })).textContent?.trim()).toBe(
    "PrismForge cannot tell whether your lights change at system start, because it cannot read its start entry.",
  );
});

test("an_entry_for_every_user_is_explained_with_its_removal", async () => {
  fakeShell(null, { lifecycle: () => lifecycleMachineEntry });
  render(App);
  await openSettings();

  expect(
    await screen.findByText(
      "A start entry for every user of this computer exists. An earlier PrismForge that ran as administrator added it, and PrismForge starts from it whatever this switch shows. PrismForge cannot remove it without administrator rights.",
    ),
  ).toBeTruthy();
  expect(screen.getByText('reg delete "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run" /v PrismForge /f')).toBeTruthy();
});

test("the_close_choice_can_be_reached_by_tab_when_background_is_stored_but_there_is_no_tray", async () => {
  fakeShell(null, { lifecycle: () => ({ ...lifecycleChosen, tray: lifecycleNoTray.tray }) });
  render(App);
  await openSettings();

  await waitFor(() => expect(radio("Quit PrismForge").getAttribute("tabindex")).toBe("0"));
  expect(radio("Keep PrismForge running in the system tray").getAttribute("tabindex")).toBe("-1");
});

test("each_switch_states_its_consequence_in_one_line_with_more_behind_details", async () => {
  fakeShell(null, { lifecycle: () => lifecycleStartPath });
  render(App);
  await openSettings();
  await waitFor(() => expect(toggle("Start PrismForge when you sign in").getAttribute("aria-checked")).toBe("true"));

  for (const item of screen.getAllByRole("switch")) {
    const description = document.getElementById(item.getAttribute("aria-describedby") ?? "");
    expect(description?.tagName).toBe("P");
    expect(description?.closest("details")).toBeNull();
  }
  const groups = screen.getAllByRole("region").filter((region) => region.closest("main"));
  expect(groups.map((group) => group.getAttribute("aria-label") ?? group.querySelector("h2")?.textContent)).toEqual([
    "When you close the window",
    "When PrismForge starts",
    "Effect output",
    "Where PrismForge keeps its files",
    "Quit PrismForge",
  ]);
});
