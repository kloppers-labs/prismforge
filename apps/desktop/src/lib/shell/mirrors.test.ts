import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { demoConnected, openRgbConnected } from "../../test/fixtures/sessions";
import { storeTwo, storeUnsaved } from "../../test/fixtures/stores";
import { itemIds, mirrored } from "../../test/menus";
import { connectionChip, findStrip, waitForConnection } from "../../test/shell";
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

test("the_registry_holds_only_items_that_a_menu_offers", () => {
  expect(Object.keys(MIRRORS).sort()).toEqual([...STRIP_IDS, ...DEVICE_IDS].sort());
});
