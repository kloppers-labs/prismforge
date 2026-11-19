import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { IdentifyOutcome } from "../../lib/api/bindings/IdentifyOutcome";
import { fakeShell, refuse } from "../../test/backend";
import * as outcomes from "../../test/fixtures/outcomes";
import { effectHandlers } from "../../test/effects";
import { demoConnected, demoEffectsRunning, demoIdentifying, openRgbConnected, openRgbDegraded } from "../../test/fixtures/sessions";
import { storeEmpty } from "../../test/fixtures/stores";

async function select(name: string, index = 0) {
  const list = await screen.findByRole("navigation", { name: /Devices/ });
  const button = within(list).getAllByRole("button", { name: new RegExp(`^${name}`) })[index];
  if (!button) {
    throw new Error(`no device ${name}`);
  }
  await fireEvent.click(button);
}

function identifyButtons() {
  return screen.getAllByRole("button", { name: /^Identify / }) as HTMLButtonElement[];
}

test("disables_unsupported_controls", async () => {
  const shell = fakeShell(openRgbDegraded, { identify: () => outcomes.restored });
  render(App);

  await select("Fixture Graphics Card");
  expect(identifyButtons().every((button) => button.disabled)).toBe(true);
  expect(screen.getByText(/Identify is off\. This device has no Direct Mode/)).toBeTruthy();
  expect(screen.getByText(/No\. PrismForge cannot set colours on this device/)).toBeTruthy();

  await select("Fixture Fan", 1);
  expect(identifyButtons().every((button) => button.disabled)).toBe(true);
  expect(screen.getByText(/Identify is off\. PrismForge cannot tell this device apart/)).toBeTruthy();

  await select("Fixture Light Bar");
  expect(identifyButtons().every((button) => button.disabled)).toBe(true);
  expect(screen.getByText(/Identify is off\. OpenRGB does not report this device right now/)).toBeTruthy();

  await select("Fixture Strip");
  const enabled = identifyButtons();
  expect(enabled.map((button) => button.disabled)).toEqual([false, false]);
  expect(screen.queryByText(/Identify is off/)).toBeNull();

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));
  expect(shell.commands().filter((command) => command === "identify")).toHaveLength(1);
});

test("explains_an_ambiguous_device_and_keeps_it_listed", async () => {
  fakeShell(openRgbDegraded);
  render(App);

  const list = await screen.findByRole("navigation", { name: /Devices/ });
  expect(within(list).getAllByText("Identical device")).toHaveLength(2);
  await select("Fixture Fan");

  expect(
    screen.getByText(
      "This device looks identical to another device. PrismForge will not light it until it can tell the two apart.",
    ),
  ).toBeTruthy();
  expect((screen.getByRole("button", { name: "Identify device" }) as HTMLButtonElement).disabled).toBe(true);
});

test("disables_identify_while_another_identify_runs", async () => {
  fakeShell(demoIdentifying);
  render(App);

  await select("Demo Graphics Card");

  expect(identifyButtons().every((button) => button.disabled)).toBe(true);
  expect(screen.getByText(/Identify is off\. Another identify is running/)).toBeTruthy();
});

test("explains_identify_before_first_use", async () => {
  fakeShell(demoConnected, { identify: () => outcomes.restored });
  render(App);

  await select("Demo Graphics Card");
  expect(screen.getByText("Identify lights it white for 3 seconds, then asks the demo backend to put back the earlier lighting.")).toBeTruthy();
  const caveat = screen.getByText(/PrismForge cannot see that\. PrismForge never saves to device memory\./);
  expect(caveat.closest("details:not([open])")).toBeNull();

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  await waitFor(() => expect((screen.getByText("How identify works").closest("details") as HTMLDetailsElement).open).toBe(false));
});

const expectations: [string, IdentifyOutcome, RegExp[]][] = [
  ["restored", outcomes.restored, [/OpenRGB accepted the requests to put back the earlier lighting/, /cannot confirm what the device shows now/]],
  ["unchangedStaleInventory", outcomes.unchangedStaleInventory, [/^Nothing was changed on the device\.$/, /Try again in a moment; it works after the scan finishes/]],
  ["unchangedLightRefused", outcomes.unchangedLightRefused, [/^Nothing was changed on the device\.$/, /^OpenRGB refused the request\.$/]],
  ["partlyRestoredPreviousStateUnknown", outcomes.partlyRestoredPreviousStateUnknown, [/the earlier lighting may not be back/, /did not know the earlier lighting/]],
  ["notRestoredConnectionLost", outcomes.notRestoredConnectionLost, [/was not put back\. The device may stay white/, /connection to OpenRGB was lost/]],
  ["notRestoredRefused", outcomes.notRestoredRefused, [/was not put back\. The device may stay white/, /OpenRGB refused the request to put the lighting back/]],
];

test.each(expectations)("renders_the_%s_outcome_truthfully", async (name, outcome, texts) => {
  fakeShell(openRgbConnected, { identify: () => outcome });
  render(App);
  await select("Fixture Strip");

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  const status = await screen.findByRole("status", { name: "" });
  for (const text of texts) {
    expect(await within(status).findByText(text)).toBeTruthy();
  }
  if (name !== "restored") {
    expect(status.textContent).not.toMatch(/accepted/);
  }
});

test("names_the_demo_backend_in_demo_outcomes", async () => {
  fakeShell(demoConnected, { identify: () => outcomes.restored });
  render(App);
  await select("Demo Motherboard");

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  const status = await screen.findByRole("status", { name: "" });
  expect(await within(status).findByText(/^Identify finished\. The demo backend accepted the requests/)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/OpenRGB accepted|asks OpenRGB|As OpenRGB reported/);
});

test("shows_busy_without_retrying", async () => {
  const shell = fakeShell(demoConnected, { identify: refuse({ Session: "Busy" }) });
  render(App);
  await select("Demo Motherboard");

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  expect(await screen.findByText(/PrismForge is still working on earlier requests\. Wait a moment, then try again\./)).toBeTruthy();
  await new Promise((resolve) => setTimeout(resolve, 200));
  expect(shell.commands().filter((command) => command === "identify")).toHaveLength(1);
  expect((screen.getByRole("button", { name: "Identify device" }) as HTMLButtonElement).disabled).toBe(false);
});

test("explains_identify_refused_for_a_device_another_program_controls", async () => {
  const conflict = { OutputConflict: { device: "demo-motherboard", owner: { External: { label: "OpenRGB Effects plugin" } } } };
  fakeShell(demoConnected, { identify: refuse({ Session: conflict }) });
  render(App);
  await select("Demo Motherboard");

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  expect(await screen.findByText(/^You said another program controls this device\. PrismForge sends nothing to it until you hand it back\.$/)).toBeTruthy();
});

test("sends_the_selected_zone", async () => {
  const shell = fakeShell(demoConnected, { identify: () => outcomes.restored });
  render(App);
  await select("Demo Motherboard");

  await fireEvent.click(screen.getByRole("button", { name: "Identify zone Addressable Header 1" }));

  expect(shell.calls.find((call) => call.command === "identify")?.args).toEqual({
    device: "demo-motherboard",
    zone: 1,
  });
});

test("set_lighting_opens_effects_with_this_device_chosen", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await select("Demo Graphics Card");

  await fireEvent.click(screen.getByRole("button", { name: "Set lighting" }));

  const targets = await screen.findByRole("navigation", { name: "Targets" });
  const card = within(targets).getByRole("group", { name: "Demo Graphics Card" });
  expect(within(card).getByRole("option", { name: /^Whole device/ }).getAttribute("aria-current")).toBe("true");
});

test("identical_names_are_told_apart_in_the_device_list", async () => {
  fakeShell(demoConnected);
  render(App);
  const list = await screen.findByRole("navigation", { name: /Devices/ });

  const rows = within(list).getAllByRole("button", { name: /^Demo Memory Module/ });

  expect(rows).toHaveLength(2);
  expect(rows[0]!.textContent).toContain("Simulated memory slot 1");
  expect(rows[1]!.textContent).toContain("Simulated memory slot 2");
});

test("the_device_list_is_one_tab_stop_moved_with_arrow_keys", async () => {
  fakeShell(demoConnected);
  render(App);
  const list = await screen.findByRole("navigation", { name: /Devices/ });
  const rows = within(list).getAllByRole("button");
  expect(rows.filter((row) => row.tabIndex === 0)).toHaveLength(1);

  rows[0]!.focus();
  await fireEvent.keyDown(rows[0]!, { key: "ArrowDown" });

  expect(document.activeElement).toBe(rows[1]);
  expect(rows[1]!.getAttribute("aria-current")).toBe("true");
  await fireEvent.keyDown(rows[1]!, { key: "End" });
  expect(document.activeElement).toBe(rows.at(-1));
});

test("the_device_says_what_controls_its_lighting_now", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);

  await select("Demo Graphics Card");

  expect(await screen.findByText("Runs the PrismForge effect Rainbow wave.")).toBeTruthy();
});

test("last_reported_colours_have_a_text_summary", async () => {
  fakeShell(demoConnected);
  render(App);

  await select("Demo Graphics Card");

  const colours = screen.getByRole("region", { name: "Last reported colours" });
  expect(colours.textContent).toMatch(/12 LEDs/);
});

test("zone_rows_have_short_buttons_with_full_names", async () => {
  fakeShell(demoConnected, { profile_store: () => storeEmpty });
  render(App);
  await select("Demo Motherboard");

  const identify = screen.getByRole("button", { name: "Identify zone Addressable Header 1" });
  const rename = await screen.findByRole("button", { name: "Rename zone Addressable Header 1" });

  expect(identify.textContent?.trim()).toBe("Identify");
  expect(rename.textContent?.trim()).toBe("Rename");
});

test("the_identical_device_note_promises_no_fix_the_core_does_not_keep", async () => {
  fakeShell(openRgbDegraded);
  render(App);
  await select("Fixture Fan");

  const note = screen.getByText(/looks identical to another device/).parentElement as HTMLElement;
  expect(note.textContent).not.toMatch(/disconnect the other one|unplug/i);
  expect(note.textContent).toMatch(/no way yet to tell them apart by hand/);
});
