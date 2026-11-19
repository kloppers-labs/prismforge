import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeTwo } from "../../test/fixtures/stores";
import { go, waitForConnection } from "../../test/shell";

test("the_controls_that_open_a_row's_menu_announce_shift_f10_and_text_fields_do_not", async () => {
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  const shortcut = (element: Element) => element.getAttribute("aria-keyshortcuts")?.split(" ") ?? [];

  await fireEvent.click(within(await screen.findByRole("navigation", { name: /Devices/ })).getByRole("button", { name: /^Demo Motherboard/ }));
  const zone = (await screen.findByRole("rowheader", { name: /^Front fans/ })).closest("tr") as HTMLElement;
  const buttons = within(zone).getAllByRole("button");
  expect(buttons.length).toBeGreaterThan(1);
  expect(buttons.every((button) => shortcut(button).includes("Shift+F10"))).toBe(true);
  await fireEvent.click(within(zone).getByRole("button", { name: /^Rename zone/ }));
  const field = await within(zone).findByRole("textbox");
  expect(field.hasAttribute("aria-keyshortcuts")).toBe(false);
  await waitFor(() => expect(shortcut(within(zone).getByRole("button", { name: "Save name" }))).toContain("Shift+F10"));

  await go("Diagnostics");
  const drawer = screen.getByRole("complementary", { name: "Diagnostics" });
  expect(shortcut(within(drawer).getByRole("button", { name: "Copy the Backend section" }))).toContain("Shift+F10");
  expect(within(drawer).getByRole("button", { name: "Copy as text" }).hasAttribute("aria-keyshortcuts")).toBe(false);

  await go("Effects");
  const targets = await screen.findByRole("navigation", { name: "Targets" });
  await fireEvent.click(within(targets).getAllByRole("button")[0]!);
  expect(shortcut(await screen.findByRole("radio", { name: "Gradient" }))).toContain("Shift+F10");
});
