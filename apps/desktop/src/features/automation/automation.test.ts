import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { AutomationView } from "../../lib/api/bindings/AutomationView";
import type { DeviceLightsOff } from "../../lib/api/bindings/DeviceLightsOff";
import type { RuleRun } from "../../lib/api/bindings/RuleRun";
import type { RuleView } from "../../lib/api/bindings/RuleView";
import { fakeShell, refuse } from "../../test/backend";
import {
  demoEventsFailed,
  demoNoRules,
  demoNotOnThisSystem,
  openRgbRunNotWritten,
  openRgbSleepFailed,
  openRgbAfterSleepAndWake,
  openRgbAlsoActs,
  openRgbDeletedProfile,
  openRgbMigratedConnects,
  openRgbSleepAndWake,
} from "../../test/fixtures/automations";
import * as errors from "../../test/fixtures/errors";
import { demoConnected } from "../../test/fixtures/sessions";

const PRISMFORGE_SENTENCES = ["When PrismForge starts, do nothing.", "When PrismForge connects, do nothing.", "When PrismForge quits, do nothing."];
const SYSTEM_SENTENCES = [
  "When the system goes to sleep, do nothing.",
  "When the system wakes, do nothing.",
  "When you sign out or the system shuts down or restarts, do nothing.",
];
const SENTENCES = [...PRISMFORGE_SENTENCES, ...SYSTEM_SENTENCES];

async function openAutomation() {
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: /Automation/ }));
  return screen.findByRole("heading", { level: 1, name: "Automation" });
}

function rule(sentence: RegExp | string) {
  return screen.getByRole("group", { name: sentence });
}

function rows(container: HTMLElement = document.body) {
  return within(container)
    .getAllByRole("listitem")
    .map((item) => item.querySelector(":scope > [role=group]"))
    .filter((row): row is HTMLElement => row !== null);
}

function detailsButton(row: HTMLElement) {
  return within(row).getByRole("button", { name: /^Details/ });
}

function details(row: HTMLElement) {
  return document.getElementById(detailsButton(row).getAttribute("aria-controls") ?? "") as HTMLElement;
}

function outsideDetails(element: HTMLElement) {
  return element.closest("[data-details]") === null;
}

function lastRun(row: HTMLElement) {
  return within(row).getByText(/^Last run/).closest("p") as HTMLElement;
}

function withRule(view: AutomationView, trigger: RuleView["trigger"], change: (rule: RuleView) => void): AutomationView {
  const copy = structuredClone(view);
  change(copy.rules.find((each) => each.trigger === trigger)!);
  return copy;
}

function sleepDevices(view: AutomationView, change: (devices: DeviceLightsOff[]) => void) {
  return withRule(view, "Sleeps", (sleep) => {
    const outcome = sleep.last_run!.outcome;
    if ("LightsOff" in outcome) {
      change(outcome.LightsOff.report.devices);
    }
  });
}

function called(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command);
}

test("automation_is_the_fifth_area_and_every_rule_reads_as_a_sentence_that_is_off", async () => {
  fakeShell(null, { automation: () => demoNoRules });
  render(App);
  const screens = await screen.findByRole("navigation", { name: "Main" });
  const names = within(screens)
    .getAllByRole("button")
    .map((button) => button.textContent?.trim());
  expect(names.indexOf("Automation")).toBe(names.indexOf("Profiles") + 1);

  await openAutomation();

  expect(screen.getByText("Rules run only while PrismForge runs.")).toBeTruthy();
  await screen.findByRole("region", { name: "System" });
  expect(rows().map((row) => row.getAttribute("aria-label"))).toEqual(SENTENCES);
  expect(screen.queryAllByRole("heading", { level: 2 }).map((heading) => heading.textContent?.trim())).toEqual(["PrismForge", "System"]);
  for (const sentence of SENTENCES) {
    const toggle = within(rule(sentence)).getByRole("switch");
    expect(toggle.getAttribute("aria-checked")).toBe("false");
  }
});

test("the_header_says_once_that_rules_run_only_while_prismforge_runs", async () => {
  fakeShell(null, { automation: () => demoNoRules });
  render(App);
  await openAutomation();

  await screen.findByText("Rules run only while PrismForge runs.");
  expect(screen.queryByText(/Each rule changes your lighting only after you switch it on/)).toBeNull();
  expect(screen.queryByText(/These are the rules of demo mode/)).toBeNull();
});

test("the_prismforge_events_and_the_system_events_are_two_groups", async () => {
  fakeShell(null, { automation: () => demoNoRules });
  render(App);
  await openAutomation();

  const prismforge = await screen.findByRole("region", { name: "PrismForge" });
  const system = screen.getByRole("region", { name: "System" });
  expect(rows(prismforge).map((row) => row.getAttribute("aria-label"))).toEqual(PRISMFORGE_SENTENCES);
  expect(rows(system).map((row) => row.getAttribute("aria-label"))).toEqual(SYSTEM_SENTENCES);
});

test("every_promise_is_one_short_line_and_the_full_explanation_is_in_a_closed_details", async () => {
  fakeShell(null, { automation: () => openRgbSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const short = within(sleep).getByText("PrismForge tries to switch the lights off before the PC sleeps.");
  expect(outsideDetails(short)).toBe(true);
  expect(detailsButton(sleep).getAttribute("aria-expanded")).toBe("false");
  expect(details(sleep).hidden).toBe(true);
  expect(details(sleep).hasAttribute("data-details")).toBe(true);
  await fireEvent.click(detailsButton(sleep));
  expect(detailsButton(sleep).getAttribute("aria-expanded")).toBe("true");
  expect(details(sleep).hidden).toBe(false);
  expect(details(sleep).textContent).toContain("Windows gives about two seconds and may skip the notice on low battery");
  expect(details(sleep).textContent).not.toContain("PrismForge tries to switch the lights off before the PC sleeps.");
  expect(details(sleep).textContent).toContain("Switching the lights off writes black through Direct Mode");

  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(wake).getByText("PrismForge applies the profile once devices return, within about 30 seconds.")).toBeTruthy();
  expect(details(wake).textContent).toContain("waits for OpenRGB and for the device list to stop changing");
  expect(details(wake).textContent).not.toContain("within about 30 seconds");
  expect(within(rule(/^When you sign out/)).getByText("PrismForge tries to act as the session ends.")).toBeTruthy();
  expect(within(rule("When PrismForge starts, do nothing.")).getByText("PrismForge will do this once, after it starts and connects by itself.")).toBeTruthy();
  expect(details(rule("When PrismForge starts, do nothing.")).textContent).toContain("Connect to OpenRGB when PrismForge starts");
  expect(within(rule("When PrismForge connects, do nothing.")).getByText("PrismForge will do this once each time you connect.")).toBeTruthy();
  expect(within(rule("When PrismForge quits, do nothing.")).getByText("PrismForge will act when you quit it.")).toBeTruthy();
  expect(details(rule("When PrismForge quits, do nothing.")).textContent).toContain("waits at most two seconds");
});

test("no_short_promise_is_longer_than_twelve_words", async () => {
  const { shortPromiseText } = await import("./wording");
  const triggers = ["Starts", "Connects", "Sleeps", "Wakes", "SessionEnds", "Quits"] as const;
  const actions = ["Nothing", "LightsOff", { ApplyProfile: { profile: "p" } }, "Newer"] as const;
  for (const trigger of triggers) {
    for (const action of actions) {
      expect(shortPromiseText(trigger, action).split(" ").length).toBeLessThanOrEqual(12);
    }
  }
});

test("lights_off_at_sleep_and_a_profile_at_wake_read_as_plain_sentences_that_are_on", async () => {
  fakeShell(null, { automation: () => openRgbSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(sleep).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(wake).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(sleep).queryByRole("combobox", { name: /^Profile/ })).toBeNull();
  expect(within(wake).getByRole("combobox", { name: /^Profile/ })).toBeTruthy();
});

test("switching_a_rule_on_and_choosing_its_action_are_explicit_commands", async () => {
  let current: AutomationView = demoNoRules;
  const shell = fakeShell(null, {
    automation: () => current,
    set_rule: () => current,
  });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, do nothing."));
  await fireEvent.change(within(sleep).getByRole("combobox", { name: /^Action/ }), { target: { value: "LightsOff" } });
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(1));
  expect(called(shell, "set_rule")[0]!.args).toEqual({ trigger: "Sleeps", enabled: false, action: "LightsOff" });

  await fireEvent.click(within(sleep).getByRole("switch"));
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(2));
  expect(called(shell, "set_rule")[1]!.args).toEqual({ trigger: "Sleeps", enabled: true, action: "Nothing" });

  const wake = rule("When the system wakes, do nothing.");
  const action = within(wake).getByRole("combobox", { name: /^Action/ }) as HTMLSelectElement;
  expect([...action.options].map((option) => option.textContent)).toEqual(["Do nothing", "Switch the lights off", "Apply a profile"]);
  await fireEvent.change(action, { target: { value: "ApplyProfile" } });
  expect(called(shell, "set_rule")).toHaveLength(2);
  const profile = within(wake).getByRole("combobox", { name: /^Profile/ }) as HTMLSelectElement;
  const glow = demoNoRules.profiles[0]!;
  await fireEvent.change(profile, { target: { value: glow.id } });
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(3));
  expect(called(shell, "set_rule")[2]!.args).toEqual({
    trigger: "Wakes",
    enabled: false,
    action: { ApplyProfile: { profile: glow.id } },
  });
});

test("the_last_run_is_one_line_with_its_time_and_result_and_each_skipped_device_stays_visible", async () => {
  fakeShell(null, { automation: () => openRgbAfterSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const run = openRgbAfterSleepAndWake.rules.find((rule) => rule.trigger === "Sleeps")!.last_run!;
  const time = new Date(run.at_ms).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
  const line = lastRun(sleep);
  expect(line.textContent?.replace(/\s+/g, " ").trim()).toBe(`Last run ${time} · Switched the lights off on 1 device.`);
  expect(outsideDetails(line)).toBe(true);
  expect(line.querySelector("svg")).toBeNull();
  const skipped = within(sleep).getByText("Skipped Fixture Keyboard: it has no Direct Mode and no “Off” mode that leaves its memory alone.");
  expect(outsideDetails(skipped)).toBe(true);
  expect(within(sleep).queryByRole("region", { name: "Last run" })).toBeNull();
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(lastRun(wake).textContent).toContain("Evening is applied.");
});

test("a_failed_last_run_shows_a_failure_icon_and_its_reason_in_the_row", async () => {
  fakeShell(null, { automation: () => openRgbSleepFailed, mark_runs_seen: () => openRgbSleepFailed });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const line = lastRun(sleep);
  expect(line.textContent).toContain("Stopped after 1 s, the time this event allows.");
  expect(line.textContent).not.toContain("Did not finish");
  expect(outsideDetails(line)).toBe(true);
  expect(line.querySelector("svg")).not.toBeNull();
});

test("devices_left_to_other_programs_or_not_there_are_counted_in_the_row_and_named_in_details", async () => {
  const view = sleepDevices(openRgbAfterSleepAndWake, (devices) => {
    devices.push(
      { device: "dev-x", name: "Case Fans", outcome: { Skipped: { reason: { DeclaredExternal: { label: "iCUE" } } } } },
      { device: "dev-y", name: "Mouse", outcome: { Skipped: { reason: "NotAvailable" } } },
    );
  });
  fakeShell(null, { automation: () => view });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  expect(lastRun(sleep).textContent).toContain("2 devices left as they are, named in Details.");
  expect(outsideDetails(within(sleep).getByText("Skipped Case Fans: you said iCUE controls it."))).toBe(false);
  expect(outsideDetails(within(sleep).getByText("Skipped Mouse: it was not there."))).toBe(false);
  const problem = within(sleep).getByText("Skipped Fixture Keyboard: it has no Direct Mode and no “Off” mode that leaves its memory alone.");
  expect(outsideDetails(problem)).toBe(true);
});

test("one_device_left_alone_is_counted_in_the_singular", async () => {
  const view = sleepDevices(openRgbAfterSleepAndWake, (devices) => {
    devices.push({ device: "dev-x", name: "Case Fans", outcome: { Skipped: { reason: { DeclaredExternal: { label: "iCUE" } } } } });
  });
  fakeShell(null, { automation: () => view });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  expect(lastRun(sleep).textContent).toContain("1 device left as it is, named in Details.");
});

test("a_lights_off_run_whose_planned_devices_were_all_left_to_other_programs_is_not_a_failure", async () => {
  const declared = sleepDevices(openRgbAfterSleepAndWake, (devices) => {
    for (const device of devices) {
      if (device.outcome === "Black") {
        device.outcome = { Skipped: { reason: { DeclaredExternal: { label: "iCUE" } } } };
      }
    }
  });
  const shell = fakeShell(null, { automation: () => declared, mark_runs_seen: () => declared });
  render(App);
  await waitFor(() => expect(called(shell, "automation").length).toBeGreaterThan(0));
  await new Promise((resolve) => setTimeout(resolve, 50));
  expect(screen.queryByRole("status", { name: "Automation" })).toBeNull();

  await openAutomation();
  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const line = lastRun(sleep);
  expect(line.querySelector("svg")).toBeNull();
  expect(line.textContent).not.toContain("Did not finish");
  expect(line.textContent).toContain("1 device left as it is, named in Details.");
});

test("a_lights_off_run_that_came_too_late_says_it_did_not_finish_on_the_run_line", async () => {
  const late = sleepDevices(openRgbAfterSleepAndWake, (devices) => {
    for (const device of devices) {
      if (device.outcome === "Black") {
        device.outcome = { Skipped: { reason: "TooLate" } };
      }
    }
  });
  fakeShell(null, { automation: () => late });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const line = lastRun(sleep);
  expect(line.querySelector("svg")).not.toBeNull();
  expect(line.textContent).toContain("Did not finish: Switched the lights off on 0 devices.");
  expect(outsideDetails(within(sleep).getByText("Skipped Fixture Strip: the event was over before PrismForge came to it."))).toBe(true);
});

test("a_profile_applied_as_prismforge_ends_explains_the_kept_frame_in_details", async () => {
  const profile = openRgbSleepAndWake.profiles[0]!;
  const view = withRule(openRgbSleepAndWake, "Quits", (rule) => {
    rule.action = { ApplyProfile: { profile: profile.id } };
    rule.profile = { Ready: { name: profile.name } };
  });
  fakeShell(null, { automation: () => view });
  render(App);
  await openAutomation();

  const quits = await waitFor(() => rule(`When PrismForge quits, apply the profile ${profile.name}.`));
  expect(details(quits).textContent).toContain("each device keeps the frame it showed last.");
});

test("a_rule_whose_profile_was_deleted_says_so_and_never_names_another_profile", async () => {
  fakeShell(null, { automation: () => openRgbDeletedProfile });
  render(App);
  await openAutomation();

  const wake = await waitFor(() => rule("When the system wakes, apply a profile that was deleted."));
  expect(within(wake).queryByRole("alert")).toBeNull();
  const problem = within(wake).getByText(
    "The profile of this rule was deleted, so the rule applies nothing. Choose another profile or switch the rule off.",
  );
  expect(outsideDetails(problem)).toBe(true);
});

test("the_screen_warns_in_one_line_when_openrgb_also_acts_on_the_same_event", async () => {
  fakeShell(null, { automation: () => openRgbAlsoActs });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const warning = within(sleep).getByText("OpenRGB also loads “Dark” when the system goes to sleep; whichever writes last wins.");
  expect(outsideDetails(warning)).toBe(true);
  expect(details(sleep).textContent).not.toContain("OpenRGB also loads");
  expect(details(sleep).textContent).toContain("Turn PrismForge's rule or OpenRGB's profile setting off to choose which one acts.");
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(wake).getByText("OpenRGB also loads “Day” when the system wakes; whichever writes last wins.")).toBeTruthy();
});

test("the_openrgb_line_of_a_quits_rule_names_the_moment_openrgb_acts", async () => {
  const quits = withRule(openRgbAlsoActs, "Quits", (rule) => {
    rule.openrgb_also = [
      { setting: "Exit", profile: "Night" },
      { setting: "ServiceShutdown", profile: "Night" },
    ];
  });
  fakeShell(null, { automation: () => quits });
  render(App);
  await openAutomation();

  const row = await waitFor(() => rule("When PrismForge quits, do nothing."));
  expect(within(row).getByText("OpenRGB also loads “Night” when OpenRGB closes; whichever writes last wins.")).toBeTruthy();
  expect(within(row).getByText("OpenRGB also loads “Night” when the OpenRGB service stops; whichever writes last wins.")).toBeTruthy();
  expect(screen.queryByText(/on this event/)).toBeNull();
});

test("without_session_events_on_this_system_the_system_group_says_so_once_and_its_rules_are_off", async () => {
  fakeShell(null, { automation: () => demoNotOnThisSystem });
  render(App);
  await openAutomation();

  const system = await screen.findByRole("region", { name: "System" });
  expect(within(system).getByText("Works on Windows only for now.")).toBeTruthy();
  expect(screen.getAllByText(/Windows only/)).toHaveLength(1);
  expect(screen.queryByText(/Not on this system/)).toBeNull();
  for (const row of rows(system)) {
    expect((within(row).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect((within(row).getByRole("combobox", { name: /^Action/ }) as HTMLSelectElement).disabled).toBe(true);
  }
  const start = rule("When PrismForge starts, do nothing.");
  expect((within(start).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
});

test("limited_support_keeps_its_note_once_in_the_system_group_and_the_rules_stay_usable", async () => {
  const limited = { ...structuredClone(demoNoRules), support: { Limited: { detail: "Session end could not be registered." } } };
  fakeShell(null, { automation: () => limited });
  render(App);
  await openAutomation();

  const system = await screen.findByRole("region", { name: "System" });
  expect(within(system).getByText("Part of PrismForge's set-up for system events failed. Session end could not be registered.")).toBeTruthy();
  expect(screen.getAllByText(/Session end could not be registered/)).toHaveLength(1);
  expect((within(rule("When the system goes to sleep, do nothing.")).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
});

test("the_openrgb_check_is_one_line_at_the_bottom_of_the_screen", async () => {
  fakeShell(null, { automation: () => demoNoRules });
  render(App);
  await openAutomation();

  const system = await screen.findByRole("region", { name: "System" });
  const check = screen.getByText(/PrismForge cannot check OpenRGB's own profile settings here/);
  expect(system.compareDocumentPosition(check) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
});

test("the_marked_profile_of_an_earlier_version_is_now_the_connects_rule", async () => {
  fakeShell(null, { automation: () => openRgbMigratedConnects });
  render(App);
  await openAutomation();

  const connects = await waitFor(() => rule("When PrismForge connects, apply the profile Evening."));
  expect(within(connects).getByRole("switch").getAttribute("aria-checked")).toBe("true");
});

test("a_refused_rule_change_is_shown_and_the_rule_keeps_the_state_rust_reports", async () => {
  fakeShell(null, {
    automation: () => demoNoRules,
    set_rule: refuse(errors.settingsInUse),
  });
  render(App);
  await openAutomation();

  const quit = await waitFor(() => rule("When PrismForge quits, do nothing."));
  await fireEvent.click(within(quit).getByRole("switch"));
  await screen.findByText(/Another PrismForge window is using the profile folder/);
  expect(within(quit).getByRole("switch").getAttribute("aria-checked")).toBe("false");
});

test("a_new_rule_run_reloads_the_rules", async () => {
  let loads = 0;
  const shell = fakeShell(demoConnected, {
    automation: () => {
      loads += 1;
      return demoNoRules;
    },
  });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });
  await waitFor(() => expect(shell.commands()).toContain("view"));
  await openAutomation();
  await waitFor(() => expect(loads).toBeGreaterThan(0));
  await new Promise((resolve) => setTimeout(resolve, 50));
  const before = loads;

  const run: RuleRun = { trigger: "Sleeps", at_ms: 1, outcome: { Failed: { failure: "NotConnected" } }, seen: true };
  await shell.pushView({ ...shell.view(demoConnected), rule_run: run });

  await waitFor(() => expect(loads).toBe(before + 1));
});

test("each_row_is_named_by_its_sentence_and_its_controls_by_the_event", async () => {
  fakeShell(null, { automation: () => openRgbSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await screen.findByRole("group", { name: "When the system goes to sleep, switch the lights off." });
  expect(within(sleep).getByRole("switch", { name: "When the system goes to sleep" })).toBeTruthy();
  expect(within(sleep).queryByText("On")).toBeNull();
  expect(screen.queryAllByRole("heading", { name: /^When / })).toHaveLength(0);
  const wake = screen.getByRole("group", { name: "When the system wakes, apply the profile Evening." });
  expect(within(wake).getByRole("switch", { name: "When the system wakes" })).toBeTruthy();
  expect(within(wake).getByRole("combobox", { name: "Action when the system wakes" })).toBeTruthy();
  expect(within(wake).getByRole("combobox", { name: "Profile when the system wakes" })).toBeTruthy();
});

test("a_seam_that_failed_to_start_is_shown_and_the_system_event_rules_are_off", async () => {
  fakeShell(null, { automation: () => demoEventsFailed });
  render(App);
  await openAutomation();

  const system = await screen.findByRole("region", { name: "System" });
  const failed = within(system).getByText(/PrismForge could not start receiving sleep, wake and session end/);
  expect(failed.textContent).toContain("the event window could not be created: Access is denied.");
  expect(screen.getAllByText(/could not start receiving/)).toHaveLength(1);
  for (const row of rows(system)) {
    expect((within(row).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
    expect((within(row).getByRole("combobox", { name: /^Action/ }) as HTMLSelectElement).disabled).toBe(true);
  }
  expect(screen.queryByText(/Windows only/)).toBeNull();
  expect((within(rule("When PrismForge quits, do nothing.")).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
});

test("a_run_the_store_could_not_write_is_shown", async () => {
  fakeShell(null, { automation: () => openRgbRunNotWritten });
  render(App);
  await openAutomation();

  expect(await screen.findByText(/PrismForge could not save rule-runs.json/)).toBeTruthy();
});

test("a_failed_sleep_run_is_a_notice_at_start_until_the_user_dismisses_it", async () => {
  let current = openRgbSleepFailed;
  const shell = fakeShell(null, {
    automation: () => current,
    mark_runs_seen: (args) => {
      expect(args).toEqual({ triggers: ["Sleeps"] });
      current = structuredClone(openRgbSleepFailed);
      current.rules.forEach((each) => each.last_run && (each.last_run.seen = true));
      return current;
    },
  });
  render(App);

  const notice = await screen.findByRole("status", { name: "Automation" });
  expect(notice.textContent).toContain("When the system went to sleep, the rule did not do what it should:");
  expect(notice.textContent).toContain("Stopped after 1 s, the time this event allows.");
  await fireEvent.click(within(notice).getByRole("button", { name: /Dismiss/ }));

  await waitFor(() => expect(called(shell, "mark_runs_seen")).toHaveLength(1));
  await waitFor(() => expect(screen.queryByRole("status", { name: "Automation" })).toBeNull());
});

test("opening_automation_marks_a_failed_run_seen", async () => {
  const shell = fakeShell(null, {
    automation: () => openRgbSleepFailed,
    mark_runs_seen: () => openRgbSleepFailed,
  });
  render(App);
  await openAutomation();

  await waitFor(() => expect(called(shell, "mark_runs_seen").map((call) => call.args)).toEqual([{ triggers: ["Sleeps"] }]));
});

test("a_command_refused_while_the_lights_go_off_says_so", async () => {
  const { failureText } = await import("../../lib/errors");
  expect(failureText("RuleRunning")).toBe("An automation rule is switching the lights off. Try again in a moment.");
});

test("a_sleep_rule_whose_lights_off_came_too_late_counts_as_failed", async () => {
  const { runFailed } = await import("./wording");
  const late: RuleRun = {
    trigger: "Sleeps",
    at_ms: 1,
    seen: false,
    outcome: {
      LightsOff: {
        report: {
          devices: [
            { device: "dev-a", name: "Strip", outcome: { Skipped: { reason: "TooLate" } } },
            { device: "dev-k", name: "Keyboard", outcome: { Skipped: { reason: "NoDirectOrOffMode" } } },
          ],
        },
      },
    },
  };
  expect(runFailed(late)).toBe(true);
  const nothingPlanned: RuleRun = {
    ...late,
    outcome: { LightsOff: { report: { devices: [{ device: "dev-k", name: "Keyboard", outcome: { Skipped: { reason: "NoDirectOrOffMode" } } }] } } },
  };
  expect(runFailed(nothingPlanned)).toBe(false);
  const declared = { Skipped: { reason: { DeclaredExternal: { label: "iCUE" } } } } as const;
  const leftToOthers: RuleRun = {
    ...late,
    outcome: { LightsOff: { report: { devices: [{ device: "dev-a", name: "Strip", outcome: declared }] } } },
  };
  expect(runFailed(leftToOthers)).toBe(false);
  const leftAndLate: RuleRun = {
    ...late,
    outcome: {
      LightsOff: {
        report: {
          devices: [
            { device: "dev-a", name: "Strip", outcome: declared },
            { device: "dev-b", name: "Fans", outcome: { Skipped: { reason: "TooLate" } } },
          ],
        },
      },
    },
  };
  expect(runFailed(leftAndLate)).toBe(true);
});

test("a_failed_sleep_run_that_was_not_recorded_still_raises_the_start_notice", async () => {
  const shell = fakeShell(demoConnected, { automation: () => demoNoRules });
  render(App);
  await screen.findByRole("navigation", { name: "Main" });
  const run: RuleRun = { trigger: "Sleeps", at_ms: 7, seen: false, outcome: { Failed: { failure: { TimedOut: { limit_ms: 1000 } } } } };

  await shell.pushView({ ...shell.view(demoConnected), rule_run: run });

  const notice = await screen.findByRole("status", { name: "Automation" });
  expect(notice.textContent).toContain("When the system went to sleep, the rule did not do what it should:");
  await fireEvent.click(within(notice).getByRole("button", { name: /Dismiss/ }));
  await waitFor(() => expect(screen.queryByRole("status", { name: "Automation" })).toBeNull());
});
