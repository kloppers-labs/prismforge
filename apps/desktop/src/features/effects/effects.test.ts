import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { RestoreOutcome } from "../../lib/api/bindings/RestoreOutcome";
import { fakeShell, refuse } from "../../test/backend";
import { findDemoMarker, go } from "../../test/shell";
import { button, chooseTarget, effectHandlers, openEffects, wait } from "../../test/effects";
import * as errors from "../../test/fixtures/errors";
import {
  demoConnected,
  demoEffectIdentifying,
  demoEffectsNotConnected,
  demoEffectsRunning,
  openRgbDegraded,
} from "../../test/fixtures/sessions";
import * as stops from "../../test/fixtures/stops";
import { lifecycleChosen, lifecycleNoTray, lifecycleQuitChosen } from "../../test/fixtures/lifecycles";

function commandsNamed(shell: ReturnType<typeof fakeShell>, name: string) {
  return shell.calls.filter((call) => call.command === name);
}

async function chooseEffect(name: string) {
  await fireEvent.click(screen.getByRole("radio", { name }));
}

async function type(input: HTMLElement, value: string) {
  await fireEvent.input(input, { target: { value } });
}

test("renders_controls_from_the_definitions_of_each_effect", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  await chooseEffect("Solid colour");
  expect(screen.getByRole("textbox", { name: "Colour" })).toBeTruthy();
  expect(screen.getByRole("slider", { name: "Brightness" })).toBeTruthy();
  expect(screen.getByRole("spinbutton", { name: "Brightness" })).toBeTruthy();
  expect(screen.queryByRole("group", { name: "Colours" })).toBeNull();

  await chooseEffect("Gradient");
  const palette = screen.getByRole("group", { name: "Colours" });
  expect(within(palette).getAllByRole("textbox")).toHaveLength(2);
  expect(within(palette).getAllByRole("textbox").map((field) => (field as HTMLInputElement).value)).toContain("#ff0040");
  expect(screen.getByRole("radio", { name: /^Forward/ })).toBeTruthy();
  expect(screen.queryByRole("textbox", { name: "Colour" })).toBeNull();

  await chooseEffect("Moving highlight");
  expect(screen.getByRole("textbox", { name: "Highlight" })).toBeTruthy();
  expect(screen.getByRole("textbox", { name: "Background" })).toBeTruthy();
  expect(screen.getByRole("spinbutton", { name: "Width" })).toBeTruthy();
  expect(screen.getByRole("spinbutton", { name: "Speed" })).toBeTruthy();
  expect(screen.getAllByText("cycles per minute").length).toBeGreaterThan(0);
});

test("applies_the_instance_the_user_configured", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Motherboard", "Addressable Header 1");
  await chooseEffect("Breathing");

  await type(screen.getByRole("textbox", { name: "Colour" }), "#00ff80");
  await type(screen.getByRole("spinbutton", { name: "Speed" }), "30");
  await fireEvent.click(button("Apply effect"));

  expect(commandsNamed(shell, "apply_effect").map((call) => call.args)).toEqual([
    {
      target: { device: "demo-motherboard", zone: 1 },
      effect: {
        effect: "breathing",
        values: {
          color: { Color: { r: 0, g: 255, b: 128 } },
          speed: { Number: 30 },
          brightness: { Number: 100 },
        },
      },
    },
  ]);
  expect(await screen.findByText(/^PrismForge accepted Breathing for Addressable Header 1\./)).toBeTruthy();
});

test("palette_colours_can_be_added_moved_and_removed_within_the_limits", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await chooseEffect("Colour cycle");
  const palette = screen.getByRole("group", { name: "Colours" });
  const count = () => within(palette).getAllByRole("textbox").length;
  const initial = count();

  await fireEvent.click(within(palette).getByRole("button", { name: "Add colour" }));
  expect(count()).toBe(initial + 1);
  const first = (within(palette).getByRole("textbox", { name: "Colour 1" }) as HTMLInputElement).value;
  await fireEvent.click(within(palette).getByRole("button", { name: "Move colour 1 down" }));
  expect((within(palette).getByRole("textbox", { name: "Colour 2" }) as HTMLInputElement).value).toBe(first);
  while (count() > 2) {
    await fireEvent.click(within(palette).getByRole("button", { name: `Remove colour ${count()}` }));
  }
  expect((within(palette).getByRole("button", { name: "Remove colour 1" }) as HTMLButtonElement).disabled).toBe(true);
  expect(within(palette).getByText(/2 to 8 colours/)).toBeTruthy();

  await fireEvent.click(button("Apply effect"));
  const sent = commandsNamed(shell, "apply_effect")[0]?.args as { effect: { values: Record<string, unknown> } };
  const colours = sent.effect.values.palette as { Palette: unknown[] };
  expect(colours.Palette).toHaveLength(2);
});

test("shows_a_rust_validation_error_at_its_control", async () => {
  fakeShell(demoConnected, effectHandlers({ apply_effect: refuse(errors.invalidBreathing) }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await chooseEffect("Breathing");
  await type(screen.getByRole("textbox", { name: "Colour" }), "#ff8000");

  await fireEvent.click(button("Apply effect"));

  expect(await screen.findByRole("spinbutton", { name: "Speed", description: /Use a value from 0 to 120\./ })).toBeTruthy();
  expect(screen.getByRole("spinbutton", { name: "Brightness", description: /Use a value from 0 to 100\./ })).toBeTruthy();
  expect(screen.getByText(/Some effect settings are not valid/)).toBeTruthy();
});

test("checks_values_on_the_client_before_sending", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await chooseEffect("Solid colour");

  await type(screen.getByRole("textbox", { name: "Colour" }), "");
  expect(screen.getByRole("textbox", { name: "Colour", description: /Required/ })).toBeTruthy();
  expect(button("Apply effect").disabled).toBe(true);
  await type(screen.getByRole("textbox", { name: "Colour" }), "orange");
  expect(screen.getByRole("textbox", { name: "Colour", description: /six hex digits/ })).toBeTruthy();
  await type(screen.getByRole("textbox", { name: "Colour" }), "#ff8000");
  await type(screen.getByRole("spinbutton", { name: "Brightness" }), "140");
  expect(screen.getByRole("spinbutton", { name: "Brightness", description: /Use a value from 0 to 100\./ })).toBeTruthy();
  expect(button("Apply effect").disabled).toBe(true);
  expect(commandsNamed(shell, "apply_effect")).toHaveLength(0);
});

test("explains_the_hand_over_on_a_conflict", async () => {
  fakeShell(demoConnected, effectHandlers({ apply_effect: refuse(errors.outputConflict) }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await chooseEffect("Rainbow wave");

  await fireEvent.click(button("Apply effect"));

  const alert = await screen.findByRole("alert");
  expect(within(alert).getByText(/^You said OpenRGB Effects plugin controls this device\./)).toBeTruthy();
  expect(within(alert).getByText(/First stop this device's effect in OpenRGB Effects plugin\./)).toBeTruthy();
  expect(within(alert).getByText(/the lights flicker or the last writer wins/)).toBeTruthy();
  expect(alert.textContent).not.toMatch(/could not run|failed/i);
});

test("disables_incompatible_targets_with_their_reasons", async () => {
  fakeShell(openRgbDegraded, effectHandlers());
  render(App);
  const targets = await openEffects();

  for (const fan of within(targets).getAllByRole("group", { name: "Fixture Fan" })) {
    expect(within(fan).getAllByRole("button").every((item) => (item as HTMLButtonElement).disabled)).toBe(true);
    expect(within(fan).getByText(/cannot tell this device apart from an identical one/)).toBeTruthy();
  }

  await chooseTarget("Fixture Graphics Card");
  expect(button("Apply effect").disabled).toBe(true);
  expect(screen.getByText(/^Effects are off\. This device has no Direct Mode/)).toBeTruthy();

  await chooseTarget("Fixture Light Bar");
  expect(button("Apply effect").disabled).toBe(true);
  expect(screen.getByText(/^Effects are off\. OpenRGB does not report this device right now\./)).toBeTruthy();
});

test("disables_a_declared_device_and_a_device_in_identify", async () => {
  fakeShell(demoEffectIdentifying, effectHandlers());
  render(App);

  await chooseTarget("Demo Memory Module", "Whole device", 1);
  expect(button("Apply effect").disabled).toBe(true);
  expect(screen.getAllByRole("button", { name: /^Use / }).every((item) => (item as HTMLButtonElement).disabled)).toBe(true);
  expect(screen.getByText(/^Effects are off\. You said OpenRGB Effects plugin controls this device\./)).toBeTruthy();

  await chooseTarget("Demo Graphics Card");
  expect(button("Apply effect").disabled).toBe(true);
  expect(button("Stop effect").disabled).toBe(true);
  expect(screen.getByText(/^Effects are off\. An identify is running on this device\./)).toBeTruthy();
});

test("disables_effects_while_not_connected", async () => {
  fakeShell(demoEffectsNotConnected, effectHandlers());
  render(App);

  await chooseTarget("Demo Graphics Card");

  expect(button("Apply effect").disabled).toBe(true);
  expect(screen.getByText(/^Effects are off\. PrismForge is not connected\./)).toBeTruthy();
  expect(screen.getByText(/^Paused\. PrismForge is not connected\./)).toBeTruthy();
});

const stopExpectations: [string, RestoreOutcome | null, RegExp[]][] = [
  ["restored", stops.stopRestored, [/^Effect stopped\. The demo backend accepted the requests to put back the lighting from before the effect\./]],
  ["other zones still run", stops.stopOtherZonesStillRun, [/^Effect stopped\. Other PrismForge effects still run on this device/]],
  ["previous state unknown", stops.stopPreviousStateUnknown, [/^Effect stopped, but the earlier lighting may not be back\./, /did not know the earlier lighting/]],
  ["not restored after a loss", stops.stopNotRestoredConnectionLost, [/^Effect stopped\. The earlier lighting was not put back\./, /connection to the demo backend was lost/]],
  ["not restored after a refusal", stops.stopNotRestoredRefused, [/^Effect stopped\. The earlier lighting was not put back\./, /refused the request to put the lighting back/]],
];

test.each(stopExpectations)("shows_the_stop_outcome_%s_truthfully", async (name, outcome, texts) => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ stop_effect: () => outcome }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  await fireEvent.click(button("Stop effect"));

  expect(commandsNamed(shell, "stop_effect")[0]?.args).toEqual({ target: { device: "demo-graphics-card", zone: null } });
  const status = await screen.findByRole("status", { name: "Effect result" });
  for (const text of texts) {
    expect(await within(status).findByText(text)).toBeTruthy();
  }
  if (name !== "restored") {
    expect(status.textContent).not.toMatch(/accepted/);
  }
});

test("shows_owner_and_output_condition_from_the_snapshot", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  const targets = await openEffects();

  const owner = (device: string, index = 0) =>
    (within(targets).getAllByRole("group", { name: device })[index] as HTMLElement).querySelector(".tag")?.textContent;
  expect(owner("Demo Graphics Card")).toBe("Effect running");
  expect(owner("Demo Memory Module")).toBe("Effect refused");
  expect(owner("Demo Memory Module", 1)).toBe("Left to OpenRGB Effects plugin");
  expect(owner("Demo Light Bar")).toBe("Hardware mode Breathing");
  expect(owner("Demo Motherboard")).toBe("Effect running");

  await chooseTarget("Demo Memory Module");
  expect(screen.getByText(/^Refused\. The demo backend refused the last frame\./)).toBeTruthy();
  await chooseTarget("Demo Motherboard", "Addressable Header 1");
  expect((screen.getByRole("radio", { name: "Moving highlight" }) as HTMLInputElement).checked).toBe(true);
  expect((screen.getByRole("textbox", { name: "Highlight" }) as HTMLInputElement).value).toBe("#ff4000");
  expect((screen.getByRole("radio", { name: /^Backward/ }) as HTMLInputElement).checked).toBe(true);
});

test("tells_software_effects_and_hardware_modes_apart", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);

  await chooseTarget("Demo Graphics Card");
  expect(screen.getByText(/^A PrismForge effect, computed by PrismForge and sent to the device\./)).toBeTruthy();
  expect(screen.getByText("Runs while PrismForge runs and is connected.")).toBeTruthy();
  expect(screen.getByText(/A hardware mode runs on the device itself/)).toBeTruthy();
  expect(screen.getByText(/Selecting one stops the PrismForge effect on this device/)).toBeTruthy();

  await chooseTarget("Demo Light Bar");
  expect(screen.getByText(/^Hardware mode Breathing, selected in PrismForge\. It runs on the device itself\./)).toBeTruthy();
  expect(screen.queryByText(/save to device|saved to the device/i)).toBeNull();
});

test("says_effects_stop_when_prismforge_closes_and_nothing_is_saved_where_the_user_applies", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  const apply = screen.getByText("How effects run").closest("details")?.parentElement as HTMLElement;
  expect(within(apply).getByText(/The first time you close the window, PrismForge asks whether to keep running or to quit\. Quitting stops its effects\./)).toBeTruthy();
  expect(within(apply).getByText(/PrismForge saves nothing by itself\./)).toBeTruthy();
  expect(within(apply).getByText(/Profiles are saved on this computer, not on the device\./)).toBeTruthy();
  expect(screen.queryByRole("button", { name: /save/i })).toBeNull();
});

test("keeps_the_demo_marker_on_the_effects_screen", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);

  await openEffects();

  expect(await findDemoMarker()).toBeTruthy();
});

test("shows_busy_without_retrying", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: refuse({ Session: "Busy" }) }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await chooseEffect("Rainbow wave");

  await fireEvent.click(button("Apply effect"));

  expect(await screen.findByText(/PrismForge is still working on earlier requests\. Wait a moment, then try again\./)).toBeTruthy();
  await wait(600);
  expect(commandsNamed(shell, "apply_effect")).toHaveLength(1);
  expect(button("Apply effect").disabled).toBe(false);
});

test("reapplies_a_running_effect_when_a_setting_changes", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  expect((screen.getByRole("radio", { name: "Rainbow wave" }) as HTMLInputElement).checked).toBe(true);

  await type(screen.getByRole("spinbutton", { name: "Brightness" }), "40");
  await type(screen.getByRole("spinbutton", { name: "Brightness" }), "45");
  await wait(600);

  const applied = commandsNamed(shell, "apply_effect");
  expect(applied).toHaveLength(1);
  expect((applied[0]?.args as { effect: { values: Record<string, unknown> } }).effect.values.brightness).toEqual({ Number: 45 });
  expect(commandsNamed(shell, "stop_effect")).toHaveLength(0);
});

test("selects_a_hardware_mode_and_records_declarations", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({
    select_mode: () => null,
    declare_external: () => null,
    clear_external: () => null,
  }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  await fireEvent.click(button("Use Breathing"));
  expect(await screen.findByText(/^The demo backend accepted the request to use hardware mode Breathing\./)).toBeTruthy();
  await type(screen.getByRole("textbox", { name: /Program name/ }), "OpenRGB Effects plugin");
  await fireEvent.click(button("Leave this device to another program"));
  await chooseTarget("Demo Memory Module", "Whole device", 1);
  expect(screen.getByText(/Before you hand it back, stop this device's effect in OpenRGB Effects plugin\./)).toBeTruthy();
  await fireEvent.click(button("Hand back to PrismForge"));

  expect(commandsNamed(shell, "select_mode")[0]?.args).toEqual({ device: "demo-graphics-card", mode: 2 });
  expect(commandsNamed(shell, "declare_external")[0]?.args).toEqual({ device: "demo-graphics-card", label: "OpenRGB Effects plugin" });
  expect(commandsNamed(shell, "clear_external")[0]?.args).toEqual({ device: "demo-memory-2" });
  expect(document.body.textContent).not.toMatch(/PrismForge stopped (the other|OpenRGB Effects)|exclusive/i);
});

test("sets_the_frame_rate_within_the_range_from_rust", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ set_frame_rate: () => 60 }));
  render(App);
  await openEffects();
  await go("Settings");

  expect(screen.getByText(/From 1 to 60 frames per second\./)).toBeTruthy();
  expect(screen.getByText(/Higher rates use more processor time\. OpenRGB and the device may not keep up\./)).toBeTruthy();
  await type(screen.getByRole("spinbutton", { name: "Frames per second" }), "75");
  await fireEvent.click(button("Set frame rate"));

  expect(commandsNamed(shell, "set_frame_rate")[0]?.args).toEqual({ frameRate: 75 });
  expect(await screen.findByText(/^You asked for 75 frames per second\. PrismForge set the nearest allowed rate, 60\./)).toBeTruthy();
});

test.each([
  ["background", lifecycleChosen, /Closing the window keeps PrismForge and its effects running in the background\. Quitting stops its effects\./],
  ["quit", lifecycleQuitChosen, /Closing the window quits PrismForge and stops its effects\./],
  ["no tray", lifecycleNoTray, /Closing the window quits PrismForge and stops its effects\./],
] as const)("the_effects_lifecycle_line_follows_the_close_choice_%s", async (_name, lifecycle, text) => {
  fakeShell(demoConnected, effectHandlers({ lifecycle: () => lifecycle }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  const apply = screen.getByText("How effects run").closest("details")?.parentElement as HTMLElement;
  await waitFor(() => expect(within(apply).getByText(text)).toBeTruthy());
});
