import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers, openEffects } from "../../test/effects";
import { detailsEvening } from "../../test/fixtures/profileDetails";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeNamed } from "../../test/fixtures/stores";

function handlers(extra: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
  return effectHandlers({ profile_store: () => storeNamed, profile_details: () => detailsEvening, ...extra });
}

function called(shell: ReturnType<typeof fakeShell>, command: string) {
  return shell.calls.filter((call) => call.command === command);
}

async function deviceList() {
  return screen.findByRole("navigation", { name: /Devices/ });
}

async function selectDevice(name: string) {
  const list = await deviceList();
  await waitFor(() => within(list).getByRole("button", { name: new RegExp(`^${name}`) }));
  await fireEvent.click(within(list).getByRole("button", { name: new RegExp(`^${name}`) }));
  return screen.findByRole("article", { name: new RegExp(`^${name}`) });
}

test("friendly_names_appear_on_devices_effects_and_profiles", async () => {
  fakeShell(demoConnected, handlers());
  render(App);

  const card = await selectDevice("Desk card");
  expect(within(card).getByText("Reported as Demo Graphics Card.")).toBeTruthy();
  const board = await selectDevice("Demo Motherboard");
  expect(within(board).getByRole("rowheader", { name: /Front fans/ })).toBeTruthy();
  expect(within(board).getByRole("button", { name: "Identify zone Front fans" })).toBeTruthy();

  const targets = await openEffects();
  expect(within(targets).getByRole("group", { name: "Desk card" })).toBeTruthy();
  const motherboard = within(targets).getByRole("group", { name: "Demo Motherboard" });
  expect(within(motherboard).getByRole("option", { name: /^Front fans/ })).toBeTruthy();

  const screens = screen.getByRole("navigation", { name: "Main" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Profiles" }));
  const list = await screen.findByRole("navigation", { name: "Profiles" });
  await fireEvent.click(within(list).getByRole("button", { name: /^Evening/ }));
  const article = await screen.findByRole("article", { name: "Evening" });
  const settings = within(article).getByRole("list", { name: "What this profile holds" });
  expect(settings.textContent).toMatch(/Desk card/);
  expect(settings.textContent).toMatch(/Solid colour on Front fans/);
});

test("a_device_name_is_set_from_the_keyboard_and_cleared_back_to_the_reported_name", async () => {
  const shell = fakeShell(demoConnected, handlers({ set_device_name: () => null }));
  render(App);
  const card = await selectDevice("Desk card");

  await fireEvent.click(await within(card).findByRole("button", { name: "Use the reported name" }));
  expect(called(shell, "set_device_name").map((call) => call.args)).toEqual([{ device: "demo-graphics-card", name: null }]);

  await fireEvent.click(within(card).getByRole("button", { name: "Rename device" }));
  const field = within(card).getByRole("textbox", { name: "Device name" });
  expect(document.activeElement).toBe(field);
  await fireEvent.input(field, { target: { value: "Tower card" } });
  await fireEvent.keyDown(field, { key: "Enter" });

  expect(called(shell, "set_device_name").map((call) => call.args)[1]).toEqual({ device: "demo-graphics-card", name: "Tower card" });
});

test("a_name_over_the_limit_is_refused_before_it_is_sent_and_escape_cancels", async () => {
  const shell = fakeShell(demoConnected, handlers({ set_zone_name: () => null }));
  render(App);
  const board = await selectDevice("Demo Motherboard");

  await fireEvent.click(await within(board).findByRole("button", { name: "Rename zone Onboard" }));
  const field = within(board).getByRole("textbox", { name: "Name for zone Onboard" });
  await fireEvent.input(field, { target: { value: "x".repeat(61) } });
  await fireEvent.keyDown(field, { key: "Enter" });

  expect(within(board).getByText("Use at most 60 characters.")).toBeTruthy();
  expect(field.getAttribute("aria-invalid")).toBe("true");
  expect(called(shell, "set_zone_name")).toHaveLength(0);
  await fireEvent.keyDown(field, { key: "Escape" });
  expect(within(board).queryByRole("textbox", { name: "Name for zone Onboard" })).toBeNull();
  expect(document.activeElement).toBe(within(board).getByRole("button", { name: "Rename zone Onboard" }));

  await fireEvent.click(within(board).getByRole("button", { name: "Rename zone Onboard" }));
  await fireEvent.input(within(board).getByRole("textbox", { name: "Name for zone Onboard" }), { target: { value: " Desk edge " } });
  await fireEvent.click(within(board).getByRole("button", { name: "Save name" }));
  expect(called(shell, "set_zone_name").map((call) => call.args)).toEqual([{ device: "demo-motherboard", zone: 0, name: "Desk edge" }]);
});

test("diagnostics_keep_the_reported_name", async () => {
  fakeShell(demoConnected, handlers());
  render(App);
  await selectDevice("Desk card");

  await fireEvent.click(screen.getByRole("button", { name: "Diagnostics" }));

  const drawer = screen.getByRole("complementary", { name: "Diagnostics" });
  expect(within(drawer).getAllByText("Demo Graphics Card").length).toBeGreaterThan(0);
  expect(within(drawer).getByText(/store-folder\/demo-store/)).toBeTruthy();
});
