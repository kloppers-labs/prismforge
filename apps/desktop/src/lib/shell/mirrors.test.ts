import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers, openEffects } from "../../test/effects";
import { demoConnected, demoEffectsRunning, openRgbConnected } from "../../test/fixtures/sessions";
import { storeTwo, storeUnsaved } from "../../test/fixtures/stores";
import { groupsDemo } from "../../test/fixtures/groups";
import { detailsEvening } from "../../test/fixtures/profileDetails";
import { itemIds, mirrored } from "../../test/menus";
import { componentRow, openSetup, setupHandlers } from "../../test/setup";
import { connectionChip, findStrip, go, waitForConnection } from "../../test/shell";
import { MIRRORS } from "./mirrors";

function checker() {
  const seen = new Set<string>();
  const check = async (ids: string[], reveal: () => Promise<unknown> = async () => {}) => {
    await reveal();
    for (const id of ids) {
      expect(id in MIRRORS, `${id} is in the mirror registry`).toBe(true);
      expect(mirrored(id), `${id} has a visible counterpart`).toBe(true);
      seen.add(id);
    }
  };
  return { check, seen: () => [...seen].sort() };
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
  "profile.delete",
];
const DIAGNOSTICS_IDS = ["diagnostics.copy-section", "diagnostics.copy-all"];
const SETUP_IDS = [
  "component.select",
  "component.turn",
  "component.flip",
  "component.identify",
  "component.group",
  "component.take-out",
  "component.delete",
  "outline.select",
  "outline.turn",
  "outline.delete",
];
const EFFECT_IDS = [
  "target.choose",
  "target.stop",
  "target.hand-back",
  "target.add-to-group",
  "target.modes",
  "target.leave",
  "effect.choose",
  "effect.apply",
  "group.select",
  "group.apply",
  "group.rename",
  "group.delete",
];

test("every_status_strip_menu_item_has_a_visible_counterpart_in_its_panel", async () => {
  const { check: expectMirrored, seen } = checker();
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

  expect(seen()).toEqual([...STRIP_IDS].sort());
});

test("every_devices_menu_item_has_a_visible_counterpart_on_the_devices_screen", async () => {
  const { check: expectMirrored, seen } = checker();
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

  expect(seen()).toEqual([...DEVICE_IDS].sort());
});

test("every_effects_menu_item_has_a_visible_counterpart_on_the_effects_screen", async () => {
  const { check: expectMirrored, seen } = checker();
  fakeShell(demoEffectsRunning, effectHandlers({ groups: () => groupsDemo }));
  render(App);
  const targets = await openEffects();
  const row = (device: string, index = 0) => within(within(targets).getAllByRole("group", { name: device })[index]!).getByRole("option", { name: /^Whole device/ });

  await fireEvent.click(row("Demo Memory Module", 1));
  await expectMirrored(await itemIds(row("Demo Memory Module", 1)));

  await fireEvent.click(row("Demo Graphics Card"));
  await expectMirrored(await itemIds(row("Demo Graphics Card")));
  const card = screen.getByRole("radio", { name: "Gradient" }).closest("label") as HTMLElement;
  await expectMirrored(await itemIds(card));
  const desk = (await within(targets).findByText("Desk")).closest("li") as HTMLElement;
  await expectMirrored(await itemIds(desk));

  expect(seen()).toEqual([...EFFECT_IDS].sort());
});

test("every_profiles_menu_item_has_a_visible_counterpart_on_the_profiles_screen", async () => {
  const { check: expectMirrored, seen } = checker();
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo, profile_details: () => detailsEvening });
  render(App);
  await go("Profiles");
  const list = await screen.findByRole("navigation", { name: "Profiles" });
  const evening = within(list).getByRole("button", { name: /^Evening \d/ });

  await fireEvent.click(evening);
  await screen.findByRole("article", { name: "Evening" });
  await expectMirrored(await itemIds(evening));

  expect(seen()).toEqual([...PROFILE_IDS].sort());
});

test("every_diagnostics_menu_item_has_a_visible_counterpart_in_the_drawer", async () => {
  const { check: expectMirrored, seen } = checker();
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  await go("Diagnostics");
  const drawer = screen.getByRole("complementary", { name: "Diagnostics" });
  await within(drawer).findByRole("region", { name: "Profile store" });

  for (const section of within(drawer).getAllByRole("region")) {
    await expectMirrored(await itemIds(section));
  }

  expect(seen()).toEqual([...DIAGNOSTICS_IDS].sort());
});

test("every_setup_menu_item_has_a_visible_counterpart_on_the_setup_screen", async () => {
  const { check: expectMirrored, seen } = checker();
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  const row = componentRow(list, "Graphics card");

  await fireEvent.click(row);
  await expectMirrored(await itemIds(row));
  await fireEvent.change(screen.getByRole("combobox", { name: "Outline" }), { target: { value: "Case" } });
  await fireEvent.click(screen.getByRole("button", { name: "Add the outline" }));
  const outline = within(list).getByRole("option", { name: "Case" });
  await expectMirrored(await itemIds(outline));

  expect(seen()).toEqual([...SETUP_IDS].sort());
});

test("the_registry_holds_only_items_that_a_menu_offers", () => {
  expect(Object.keys(MIRRORS).sort()).toEqual([...STRIP_IDS, ...DEVICE_IDS, ...EFFECT_IDS, ...PROFILE_IDS, ...DIAGNOSTICS_IDS, ...SETUP_IDS].sort());
});
