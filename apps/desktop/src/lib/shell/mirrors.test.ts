import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers, openEffects } from "../../test/effects";
import { demoConnected, demoEffectsRunning, openRgbConnected } from "../../test/fixtures/sessions";
import { storeTwo, storeUnsaved } from "../../test/fixtures/stores";
import { itemIds, mirrored } from "../../test/menus";
import { detailsEvening } from "../../test/fixtures/profileDetails";
import { connectionChip, findStrip, go, waitForConnection } from "../../test/shell";
import { MIRRORS } from "./mirrors";

const covered = new Set<string>();

async function expectMirrored(ids: string[], reveal: () => Promise<unknown> = async () => {}) {
  await reveal();
  for (const id of ids) {
    expect(id in MIRRORS, `${id} is in the mirror registry`).toBe(true);
    expect(mirrored(id), `${id} has a visible counterpart`).toBe(true);
    covered.add(id);
  }
}

const STRIP_IDS = ["connection.rescan", "connection.end", "connection.connect", "connection.demo", "profile-chip.save", "profile-chip.open"];
const DEVICE_IDS = [
  "device.set-lighting",
  "device.identify",
  "device.rename",
  "device.reported-name",
  "zone.identify",
  "zone.rename",
  "zone.reported-name",
];
const PROFILE_IDS = [
  "profile.apply",
  "profile.update",
  "profile.rename",
  "profile.duplicate",
  "profile.export",
  "profile.apply-on-connect",
  "profile.delete",
];
const EFFECT_IDS = ["target.choose", "target.stop", "target.hand-back", "target.modes", "target.leave", "effect.choose", "effect.apply"];

test("every_status_strip_menu_item_has_a_visible_counterpart_in_its_panel", async () => {
  covered.clear();
  const shell = fakeShell(openRgbConnected, { ...effectHandlers(), profile_store: () => storeUnsaved, disconnect: () => shell.view(null) });
  render(App);
  await waitForConnection("Connected");
  const strip = await findStrip();
  const profile = await within(strip).findByRole("button", { name: /^Profile:/ });

  await expectMirrored(await itemIds(connectionChip()), () => fireEvent.click(connectionChip()));
  await fireEvent.click(connectionChip());
  await expectMirrored(await itemIds(profile), () => fireEvent.click(profile));
  await fireEvent.click(profile);

  await shell.push(null);
  await waitForConnection("Not connected");
  await expectMirrored(await itemIds(connectionChip()), () => fireEvent.click(connectionChip()));

  expect([...covered].sort()).toEqual([...STRIP_IDS].sort());
});

test("every_devices_menu_item_has_a_visible_counterpart_on_the_devices_screen", async () => {
  covered.clear();
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await screen.findByText("Desk card");
  const list = screen.getByRole("navigation", { name: /Devices/ });

  const card = within(list).getByRole("button", { name: /^Desk card/ });
  await fireEvent.click(card);
  await expectMirrored(await itemIds(card));

  await fireEvent.click(within(list).getByRole("button", { name: /^Demo Motherboard/ }));
  const zone = (await screen.findByRole("rowheader", { name: /^Front fans/ })).closest("tr") as HTMLElement;
  await expectMirrored(await itemIds(zone));

  expect([...covered].sort()).toEqual([...DEVICE_IDS].sort());
});

test("every_effects_menu_item_has_a_visible_counterpart_on_the_effects_screen", async () => {
  covered.clear();
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  const targets = await openEffects();
  const row = (device: string, index = 0) => within(within(targets).getAllByRole("group", { name: device })[index]!).getByRole("button", { name: /^Whole device/ });

  await fireEvent.click(row("Demo Memory Module", 1));
  await expectMirrored(await itemIds(row("Demo Memory Module", 1)));

  await fireEvent.click(row("Demo Graphics Card"));
  await expectMirrored(await itemIds(row("Demo Graphics Card")));
  const card = screen.getByRole("radio", { name: "Gradient" }).closest("label") as HTMLElement;
  await expectMirrored(await itemIds(card));

  expect([...covered].sort()).toEqual([...EFFECT_IDS].sort());
});

test("every_profiles_menu_item_has_a_visible_counterpart_on_the_profiles_screen", async () => {
  covered.clear();
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo, profile_details: () => detailsEvening });
  render(App);
  await go("Profiles");
  const list = await screen.findByRole("navigation", { name: "Profiles" });
  const evening = within(list).getByRole("button", { name: /^Evening \d/ });

  await fireEvent.click(evening);
  await screen.findByRole("article", { name: "Evening" });
  await expectMirrored(await itemIds(evening));

  expect([...covered].sort()).toEqual([...PROFILE_IDS].sort());
});

test("the_registry_holds_only_items_that_a_menu_offers", () => {
  expect(Object.keys(MIRRORS).sort()).toEqual([...STRIP_IDS, ...DEVICE_IDS, ...EFFECT_IDS, ...PROFILE_IDS].sort());
});
