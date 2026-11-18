import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { IdentifyOutcome } from "../../lib/api/bindings/IdentifyOutcome";
import { fakeShell, refuse } from "../../test/backend";
import * as outcomes from "../../test/fixtures/outcomes";
import { demoConnected, demoIdentifying, openRgbConnected, openRgbDegraded } from "../../test/fixtures/sessions";

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
  expect(screen.getByText(/PrismForge cannot see that\. PrismForge never saves to device memory\./)).toBeTruthy();
  expect(screen.queryByText("How identify works")).toBeNull();

  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));

  expect(await screen.findByText("How identify works")).toBeTruthy();
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
