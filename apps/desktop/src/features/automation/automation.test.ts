import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { AutomationView } from "../../lib/api/bindings/AutomationView";
import type { RuleRun } from "../../lib/api/bindings/RuleRun";
import { fakeShell, refuse } from "../../test/backend";
import {
  demoNoRules,
  demoNotOnThisSystem,
  openRgbAfterSleepAndWake,
  openRgbAlsoActs,
  openRgbDeletedProfile,
  openRgbMigratedConnects,
  openRgbSleepAndWake,
} from "../../test/fixtures/automations";
import * as errors from "../../test/fixtures/errors";
import { demoConnected } from "../../test/fixtures/sessions";

const SENTENCES = [
  "When PrismForge starts, do nothing.",
  "When PrismForge connects, do nothing.",
  "When the system goes to sleep, do nothing.",
  "When the system wakes, do nothing.",
  "When you sign out or the system shuts down or restarts, do nothing.",
  "When PrismForge quits, do nothing.",
];

async function openAutomation() {
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: /Automation/ }));
  return screen.findByRole("heading", { level: 1, name: "Automation" });
}

function rule(sentence: RegExp | string) {
  const heading = screen.getByRole("heading", { level: 2, name: sentence });
  return heading.closest("li") as HTMLElement;
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
  const headings = await screen.findAllByRole("heading", { level: 2 });
  expect(headings.map((heading) => heading.textContent?.trim())).toEqual(SENTENCES);
  for (const sentence of SENTENCES) {
    const toggle = within(rule(sentence)).getByRole("switch");
    expect(toggle.getAttribute("aria-checked")).toBe("false");
  }
});

test("lights_off_at_sleep_and_a_profile_at_wake_read_as_plain_sentences_with_their_promises", async () => {
  fakeShell(null, { automation: () => openRgbSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(sleep).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(wake).getByRole("switch").getAttribute("aria-checked")).toBe("true");
  expect(within(sleep).getByText(/PrismForge tries to switch the lights off before the PC sleeps/)).toBeTruthy();
  expect(within(wake).getByText(/once your devices are back, within about 30 seconds/)).toBeTruthy();
  const session = rule("When you sign out or the system shuts down or restarts, do nothing.");
  expect(within(session).getByText(/PrismForge tries to/)).toBeTruthy();
  const start = rule("When PrismForge starts, do nothing.");
  expect(within(start).getByText(/PrismForge will do this while it runs/)).toBeTruthy();
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
  await fireEvent.change(within(sleep).getByRole("combobox", { name: "Action" }), { target: { value: "LightsOff" } });
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(1));
  expect(called(shell, "set_rule")[0]!.args).toEqual({ trigger: "Sleeps", enabled: false, action: "LightsOff" });

  await fireEvent.click(within(sleep).getByRole("switch"));
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(2));
  expect(called(shell, "set_rule")[1]!.args).toEqual({ trigger: "Sleeps", enabled: true, action: "Nothing" });

  const wake = rule("When the system wakes, do nothing.");
  const action = within(wake).getByRole("combobox", { name: "Action" }) as HTMLSelectElement;
  expect([...action.options].map((option) => option.textContent)).toEqual(["Do nothing", "Apply a profile"]);
  await fireEvent.change(action, { target: { value: "ApplyProfile" } });
  expect(called(shell, "set_rule")).toHaveLength(2);
  const profile = within(wake).getByRole("combobox", { name: "Profile" }) as HTMLSelectElement;
  const glow = demoNoRules.profiles[0]!;
  await fireEvent.change(profile, { target: { value: glow.id } });
  await waitFor(() => expect(called(shell, "set_rule")).toHaveLength(3));
  expect(called(shell, "set_rule")[2]!.args).toEqual({
    trigger: "Wakes",
    enabled: false,
    action: { ApplyProfile: { profile: glow.id } },
  });
});

test("the_last_run_shows_its_time_what_changed_and_each_skipped_device_with_its_reason", async () => {
  fakeShell(null, { automation: () => openRgbAfterSleepAndWake });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  const run = openRgbAfterSleepAndWake.rules.find((rule) => rule.trigger === "Sleeps")!.last_run!;
  const last = within(sleep).getByRole("region", { name: "Last run" });
  expect(last.textContent).toContain(new Date(run.at_ms).toLocaleString());
  expect(last.textContent).toContain("Switched the lights off on 1 device.");
  expect(last.textContent).toContain("Skipped Fixture Keyboard: it has no Direct Mode and no “Off” mode that leaves its memory alone.");
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(wake).getByRole("region", { name: "Last run" }).textContent).toContain("Evening is applied.");
});

test("a_rule_whose_profile_was_deleted_says_so_and_never_names_another_profile", async () => {
  fakeShell(null, { automation: () => openRgbDeletedProfile });
  render(App);
  await openAutomation();

  const wake = await waitFor(() => rule("When the system wakes, apply a profile that was deleted."));
  expect(within(wake).getByRole("alert").textContent).toBe(
    "The profile of this rule was deleted, so the rule applies nothing. Choose another profile or switch the rule off.",
  );
});

test("the_screen_warns_when_openrgb_also_acts_on_the_same_event", async () => {
  fakeShell(null, { automation: () => openRgbAlsoActs });
  render(App);
  await openAutomation();

  const sleep = await waitFor(() => rule("When the system goes to sleep, switch the lights off."));
  expect(within(sleep).getByText(/OpenRGB also loads its profile “Dark” when the system goes to sleep/)).toBeTruthy();
  expect(within(sleep).getByText(/Two programs will act; the one that writes last wins/)).toBeTruthy();
  const wake = rule("When the system wakes, apply the profile Evening.");
  expect(within(wake).getByText(/OpenRGB also loads its profile “Day” when the system wakes/)).toBeTruthy();
});

test("without_session_events_on_this_system_the_screen_says_the_rules_work_on_windows_only", async () => {
  fakeShell(null, { automation: () => demoNotOnThisSystem });
  render(App);
  await openAutomation();

  await screen.findByText(/sleep, wake and session-end rules work on Windows only for now/);
  const sleep = rule("When the system goes to sleep, do nothing.");
  expect((within(sleep).getByRole("switch") as HTMLButtonElement).disabled).toBe(true);
  const start = rule("When PrismForge starts, do nothing.");
  expect((within(start).getByRole("switch") as HTMLButtonElement).disabled).toBe(false);
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

  const run: RuleRun = { trigger: "Sleeps", at_ms: 1, outcome: { Failed: { failure: "NotConnected" } } };
  await shell.pushView({ ...shell.view(demoConnected), rule_run: run });

  await waitFor(() => expect(loads).toBe(before + 1));
});
