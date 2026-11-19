import { fireEvent, render, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { openRgbConnected } from "../../test/fixtures/sessions";
import { storeUnsaved } from "../../test/fixtures/stores";
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

test("every_status_strip_menu_item_has_a_visible_counterpart_in_its_panel", async () => {
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

test("the_registry_holds_only_items_that_a_menu_offers", () => {
  expect(Object.keys(MIRRORS).sort()).toEqual([...STRIP_IDS].sort());
});
