import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { detailsEvening } from "../../test/fixtures/profileDetails";
import { previewChanges } from "../../test/fixtures/saves";
import { demoConnected, openRgbConnected, openRgbRefused } from "../../test/fixtures/sessions";
import { storeEmpty, storeUnsaved } from "../../test/fixtures/stores";
import { itemLabels, openMenu, pick } from "../../test/menus";
import { connectionChip, current, findStrip, go, profileChip, waitForConnection } from "../../test/shell";

test("the_connection_chip_menu_mirrors_its_panel_while_connected", async () => {
  const shell = fakeShell(openRgbConnected, { ...effectHandlers(), refresh: () => null, disconnect: () => shell.view(null) });
  render(App);
  await waitForConnection("Connected");

  const menu = await openMenu(connectionChip(), "Connection actions");
  expect(itemLabels(menu)).toEqual(["Rescan devices", "Disconnect"]);
  expect(within(menu).getByRole("menuitem", { name: "Disconnect" }).className).toMatch(/danger/);
  await fireEvent.click(within(menu).getByRole("menuitem", { name: "Rescan devices" }));
  expect(shell.commands().filter((command) => command === "refresh")).toHaveLength(1);

  await pick(connectionChip(), "Connection actions", "Disconnect");
  expect(shell.commands()).toContain("disconnect");
});

test("the_connection_chip_menu_offers_connect_and_demo_mode_when_not_connected", async () => {
  const shell = fakeShell(null, { profile_store: () => storeEmpty, start_demo: () => shell.view(demoConnected), ...effectHandlers() });
  render(App);
  await findStrip();
  await go("Settings");

  const menu = await openMenu(connectionChip(), "Connection actions");
  expect(itemLabels(menu)).toEqual(["Connect to OpenRGB", "Start demo mode"]);
  await fireEvent.click(within(menu).getByRole("menuitem", { name: "Connect to OpenRGB" }));
  await waitFor(() => expect(document.activeElement?.id).toBe("connect-host"));
  expect(current()).toEqual(["Devices"]);

  await pick(connectionChip(), "Connection actions", "Start demo mode");
  expect(shell.commands()).toContain("start_demo");
});

test("a_menu_whose_only_item_ends_the_session_focuses_the_menu_not_the_item", async () => {
  const shell = fakeShell(openRgbRefused, { disconnect: () => shell.view(null) });
  render(App);
  await waitForConnection("Retrying");
  const chip = connectionChip();
  chip.focus();

  await fireEvent.keyDown(chip, { key: "F10", shiftKey: true });
  const menu = screen.getByRole("menu", { name: "Connection actions" });
  expect(itemLabels(menu)).toEqual(["Stop retrying"]);
  expect(document.activeElement).toBe(menu);
  await fireEvent.keyDown(menu, { key: "Enter" });
  expect(shell.commands()).not.toContain("disconnect");

  await fireEvent.keyDown(menu, { key: "ArrowDown" });
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Enter" });
  expect(shell.commands()).toContain("disconnect");
});

test("a_rescan_that_still_runs_stays_in_the_menu_with_its_reason", async () => {
  const shell = fakeShell(openRgbConnected, { ...effectHandlers(), refresh: () => new Promise(() => {}) });
  render(App);
  await waitForConnection("Connected");
  await pick(connectionChip(), "Connection actions", "Rescan devices");

  const menu = await openMenu(connectionChip(), "Connection actions");
  const rescan = within(menu).getByRole("menuitem", { name: "Rescan devices" });
  expect(rescan.getAttribute("aria-disabled")).toBe("true");
  expect(rescan.textContent).toMatch(/PrismForge is still reading the device list\./);
  await fireEvent.click(rescan);
  expect(shell.commands().filter((command) => command === "refresh")).toHaveLength(1);
});

test("the_profile_chip_menu_saves_changes_and_opens_profiles_like_its_panel", async () => {
  const shell = fakeShell(demoConnected, {
    ...effectHandlers(),
    profile_store: () => storeUnsaved,
    profile_details: () => detailsEvening,
    preview_profile_update: () => previewChanges,
  });
  render(App);
  const chip = await within(await findStrip()).findByRole("button", { name: /^Profile: Evening/ });

  const menu = await openMenu(chip, "Profile actions");
  expect(itemLabels(menu)).toEqual(["Save changes…", "Open Profiles"]);
  await fireEvent.click(within(menu).getByRole("menuitem", { name: "Save changes…" }));
  await waitFor(() => expect(shell.calls.find((call) => call.command === "preview_profile_update")?.args).toEqual({ id: "p-0000000000000001" }));
  expect(await screen.findByRole("dialog", { name: "Save changes to Evening?" })).toBeTruthy();
  await fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Cancel" }));

  await go("Devices");
  await pick(profileChip(), "Profile actions", "Open Profiles");
  expect(current()).toEqual(["Profiles"]);
});
