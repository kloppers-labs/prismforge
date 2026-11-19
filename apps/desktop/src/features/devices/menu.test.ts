import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import * as outcomes from "../../test/fixtures/outcomes";
import { demoConnected, demoIdentifying } from "../../test/fixtures/sessions";
import { storeTwo } from "../../test/fixtures/stores";
import { itemLabels, openMenu, pick } from "../../test/menus";

function handlers() {
  return effectHandlers({
    profile_store: () => storeTwo,
    identify: () => outcomes.restored,
    set_device_name: () => null,
    set_zone_name: () => null,
  });
}

async function row(name: string) {
  const list = await screen.findByRole("navigation", { name: /Devices/ });
  return within(list).getAllByRole("button", { name: new RegExp(`^${name}`) })[0] as HTMLButtonElement;
}

function calls(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command).map((call) => call.args);
}

test("the_device_row_menu_offers_the_page_actions_for_that_device", async () => {
  fakeShell(demoConnected, handlers());
  render(App);
  await screen.findByText("Desk card");

  expect(itemLabels(await openMenu(await row("Demo Light Bar"), "Actions for Demo Light Bar"))).toEqual([
    "Set lighting",
    "Identify device",
    "Rename device",
  ]);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(itemLabels(await openMenu(await row("Desk card"), "Actions for Desk card"))).toEqual([
    "Set lighting",
    "Identify device",
    "Rename device",
    "Use the reported name",
  ]);
});

test("identify_in_the_device_menu_selects_the_device_and_calls_identify_like_its_button", async () => {
  const shell = fakeShell(demoConnected, handlers());
  render(App);

  await pick(await row("Demo Motherboard"), "Actions for Demo Motherboard", "Identify device");

  expect(await screen.findByRole("heading", { level: 2, name: "Demo Motherboard" })).toBeTruthy();
  await waitFor(() => expect(calls(shell, "identify")).toEqual([{ device: "demo-motherboard", zone: null }]));
  await fireEvent.click(screen.getByRole("button", { name: "Identify device" }));
  await waitFor(() => expect(calls(shell, "identify")).toHaveLength(2));
  expect(calls(shell, "identify")[1]).toEqual(calls(shell, "identify")[0]);
});

test("set_lighting_in_the_device_menu_opens_effects_on_that_device", async () => {
  fakeShell(demoConnected, handlers());
  render(App);

  await pick(await row("Demo Light Bar"), "Actions for Demo Light Bar", "Set lighting");

  expect(await screen.findByRole("heading", { level: 2, name: "Demo Light Bar" })).toBeTruthy();
  expect(screen.getByRole("navigation", { name: "Targets" })).toBeTruthy();
});

test("rename_in_the_device_menu_opens_the_name_field_and_reported_name_clears_it", async () => {
  const shell = fakeShell(demoConnected, handlers());
  render(App);
  await screen.findByText("Desk card");

  await pick(await row("Desk card"), "Actions for Desk card", "Rename device");
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Device name" })));
  expect((document.activeElement as HTMLInputElement).value).toBe("Desk card");
  await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

  await pick(await row("Desk card"), "Actions for Desk card", "Use the reported name");
  await waitFor(() => expect(calls(shell, "set_device_name")).toEqual([{ device: "demo-graphics-card", name: null }]));
});

test("the_zone_row_menu_identifies_and_names_the_zone_like_its_buttons", async () => {
  const shell = fakeShell(demoConnected, handlers());
  render(App);
  await fireEvent.click(await row("Demo Motherboard"));
  const zone = (await screen.findByRole("rowheader", { name: /^Front fans/ })).closest("tr") as HTMLElement;

  expect(itemLabels(await openMenu(zone, "Actions for zone Front fans"))).toEqual(["Identify zone", "Rename zone", "Use the reported name"]);
  await fireEvent.click(screen.getByRole("menuitem", { name: "Identify zone" }));
  await waitFor(() => expect(calls(shell, "identify")).toEqual([{ device: "demo-motherboard", zone: 1 }]));

  await pick(zone, "Actions for zone Front fans", "Rename zone");
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Name for zone Front fans" })));
  await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

  await pick(zone, "Actions for zone Front fans", "Use the reported name");
  await waitFor(() => expect(calls(shell, "set_zone_name")).toEqual([{ device: "demo-motherboard", zone: 1, name: null }]));
});

test("identify_that_is_off_stays_in_the_menu_with_the_same_reason_as_its_button", async () => {
  const shell = fakeShell(demoIdentifying, handlers());
  render(App);

  await screen.findByText("Desk card");
  const menu = await openMenu(await row("Desk card"), "Actions for Desk card");
  const identify = within(menu).getByRole("menuitem", { name: "Identify device" });

  expect(identify.getAttribute("aria-disabled")).toBe("true");
  expect(identify.textContent).toMatch(/Another identify is running\. Wait until it finishes\./);
  await fireEvent.click(identify);
  expect(calls(shell, "identify")).toEqual([]);
});
