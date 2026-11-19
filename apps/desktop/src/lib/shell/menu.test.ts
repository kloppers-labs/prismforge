import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { lifecycleAsking } from "../../test/fixtures/lifecycles";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeTwo } from "../../test/fixtures/stores";
import MenuHarness from "../../test/MenuHarness.svelte";
import { menuItems } from "../../test/menus";
import { connectionChip, findStrip, go, profileChip, waitForConnection } from "../../test/shell";
import type { Menu } from "./menu.svelte";

afterEach(() => vi.unstubAllGlobals());

function harness(ran: string[] = []) {
  const menu: Menu = {
    label: "Test actions",
    items: [
      { id: "connection.rescan", label: "Alpha", reason: "Alpha is off.", run: () => ran.push("Alpha") },
      { id: "connection.end", label: "Delta", danger: true, run: () => ran.push("Delta") },
      { id: "connection.connect", label: "Bravo", run: () => ran.push("Bravo") },
      { id: "connection.demo", label: "Brick", run: () => ran.push("Brick") },
      { id: "profile-chip.open", label: "Charlie", checked: true, run: () => ran.push("Charlie") },
    ],
  };
  render(MenuHarness, { menu });
  return { origin: screen.getByRole("button", { name: "Origin" }), ran };
}

function menu() {
  return screen.getByRole("menu", { name: "Test actions" });
}

function items(): HTMLElement[] {
  return menuItems(menu());
}

function focused(): string {
  return document.activeElement?.querySelector("[id$='-label']")?.textContent ?? "";
}

async function openByKeyboard(origin: HTMLElement) {
  origin.focus();
  await fireEvent.keyDown(origin, { key: "F10", shiftKey: true });
  return menu();
}

test("a_right_click_shift_f10_and_the_menu_key_open_the_menu", async () => {
  const { origin } = harness();

  await fireEvent.contextMenu(origin);
  expect(menu()).toBeTruthy();
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(screen.queryByRole("menu")).toBeNull();

  origin.focus();
  await fireEvent.keyDown(origin, { key: "F10", shiftKey: true });
  expect(menu()).toBeTruthy();
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });

  origin.focus();
  await fireEvent.keyDown(origin, { key: "ContextMenu" });
  expect(menu()).toBeTruthy();
});

test("the_right_click_that_opens_the_menu_keeps_the_webview_menu_away", async () => {
  const { origin } = harness();

  const shown = await fireEvent.contextMenu(origin);

  expect(shown).toBe(false);
  expect(menu()).toBeTruthy();
});

test("focus_moves_to_the_first_enabled_item", async () => {
  const { origin } = harness();

  await openByKeyboard(origin);

  expect(focused()).toBe("Bravo");
});

test("destructive_items_sit_last_after_a_separator", async () => {
  const { origin } = harness();
  await openByKeyboard(origin);

  const order = items().map((item) => item.querySelector("[id$='-label']")?.textContent);
  expect(order).toEqual(["Alpha", "Bravo", "Brick", "Charlie", "Delta"]);
  const separator = within(menu()).getByRole("separator");
  expect(separator.nextElementSibling).toBe(items().at(-1));
});

test("arrow_keys_wrap_and_home_and_end_jump", async () => {
  const { origin } = harness();
  await openByKeyboard(origin);

  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "ArrowUp" });
  expect(focused()).toBe("Alpha");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "ArrowUp" });
  expect(focused()).toBe("Delta");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "ArrowDown" });
  expect(focused()).toBe("Alpha");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "End" });
  expect(focused()).toBe("Delta");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Home" });
  expect(focused()).toBe("Alpha");
});

test("type_ahead_moves_to_the_next_item_with_that_first_letter", async () => {
  const { origin } = harness();
  await openByKeyboard(origin);

  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "b" });
  expect(focused()).toBe("Brick");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "B" });
  expect(focused()).toBe("Bravo");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "d" });
  expect(focused()).toBe("Delta");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "a" });
  expect(focused()).toBe("Alpha");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "z" });
  expect(focused()).toBe("Alpha");
});

test("escape_and_tab_close_and_return_focus_to_the_origin", async () => {
  const { origin } = harness();

  await openByKeyboard(origin);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(screen.queryByRole("menu")).toBeNull();
  expect(document.activeElement).toBe(origin);

  await openByKeyboard(origin);
  const tab = await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Tab" });
  expect(tab).toBe(false);
  expect(screen.queryByRole("menu")).toBeNull();
  expect(document.activeElement).toBe(origin);
});

test("a_click_outside_closes_the_menu", async () => {
  const { origin } = harness();
  await fireEvent.contextMenu(origin);

  await fireEvent.pointerDown(screen.getByRole("button", { name: "Elsewhere" }));

  expect(screen.queryByRole("menu")).toBeNull();
});

test("disabled_items_stay_focusable_and_say_why", async () => {
  const { origin, ran } = harness();
  await openByKeyboard(origin);
  const alpha = items()[0]!;

  expect(alpha.getAttribute("aria-disabled")).toBe("true");
  expect(alpha.getAttribute("tabindex")).toBe("-1");
  const reason = document.getElementById(alpha.getAttribute("aria-describedby") ?? "");
  expect(reason?.textContent).toBe("Alpha is off.");
  expect(within(alpha).getByText("Alpha is off.")).toBeTruthy();

  await fireEvent.keyDown(alpha, { key: "Home" });
  expect(document.activeElement).toBe(alpha);
  await fireEvent.keyDown(alpha, { key: "Enter" });
  await fireEvent.click(alpha);
  expect(ran).toEqual([]);
  expect(menu()).toBeTruthy();
});

test("enter_space_and_click_run_the_item_once_and_close", async () => {
  const { origin, ran } = harness();

  await openByKeyboard(origin);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Enter" });
  expect(screen.queryByRole("menu")).toBeNull();
  expect(document.activeElement).toBe(origin);

  await openByKeyboard(origin);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "b" });
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: " " });

  await fireEvent.contextMenu(origin);
  await fireEvent.click(items().at(-1)!);

  expect(ran).toEqual(["Bravo", "Brick", "Delta"]);
});

test("checkbox_items_report_their_state", async () => {
  const { origin } = harness();
  await openByKeyboard(origin);

  const charlie = within(menu()).getByRole("menuitemcheckbox");
  expect(charlie.getAttribute("aria-checked")).toBe("true");
  expect(within(menu()).getAllByRole("menuitem")).toHaveLength(4);
});

test("a_text_field_keeps_the_platform_menu", async () => {
  harness();
  const field = screen.getByRole("textbox", { name: "Field" });

  const shown = await fireEvent.contextMenu(field);
  field.focus();
  await fireEvent.keyDown(field, { key: "F10", shiftKey: true });

  expect(shown).toBe(true);
  expect(screen.queryByRole("menu")).toBeNull();
});

test("a_menu_never_opens_inside_an_inert_layer", async () => {
  harness();

  await fireEvent.contextMenu(screen.getByRole("button", { name: "Hidden origin", hidden: true }));

  expect(screen.queryByRole("menu")).toBeNull();
});

test("a_menu_never_opens_behind_a_modal_dialog", async () => {
  fakeShell(demoConnected, { ...effectHandlers(), lifecycle: () => lifecycleAsking, profile_store: () => storeTwo });
  render(App);
  await screen.findByRole("dialog");

  await fireEvent.contextMenu(connectionChip());

  expect(screen.queryByRole("menu")).toBeNull();
});

test("opening_a_menu_closes_an_open_panel_and_opening_a_panel_closes_the_menu", async () => {
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  await within(await findStrip()).findByRole("button", { name: /^Profile:/ });

  await fireEvent.click(connectionChip());
  expect(screen.getByRole("group", { name: "Connection details" })).toBeTruthy();
  await fireEvent.contextMenu(profileChip());
  expect(screen.queryByRole("group", { name: "Connection details" })).toBeNull();
  expect(screen.getByRole("menu", { name: "Profile actions" })).toBeTruthy();

  await fireEvent.click(connectionChip());
  await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
  expect(screen.getByRole("group", { name: "Connection details" })).toBeTruthy();
});

test("a_resize_or_a_scroll_keeps_the_menu_open", async () => {
  const { origin } = harness();
  await openByKeyboard(origin);

  await fireEvent(window, new Event("resize"));
  await fireEvent.scroll(window);

  expect(menu()).toBeTruthy();
  expect(focused()).toBe("Bravo");
});

test("a_covering_diagnostics_drawer_keeps_page_menus_shut_and_its_own_menus_open", async () => {
  vi.stubGlobal("matchMedia", (query: string) => ({ matches: query.includes("max-width"), media: query, addEventListener() {}, removeEventListener() {} }));
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  const row = within(await screen.findByRole("navigation", { name: /Devices/ })).getAllByRole("button")[0]!;
  await go("Diagnostics");
  expect(screen.getByRole("main").inert).toBe(true);

  await fireEvent.contextMenu(row);
  expect(screen.queryByRole("menu")).toBeNull();

  const drawer = screen.getByRole("complementary", { name: "Diagnostics" });
  await fireEvent.contextMenu(within(drawer).getByRole("region", { name: "Backend" }));
  expect(screen.getByRole("menu", { name: "Actions for Backend" })).toBeTruthy();
});

test("a_menu_closes_when_the_item_it_came_from_goes_away", async () => {
  const shell = fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  const row = within(await screen.findByRole("navigation", { name: /Devices/ })).getAllByRole("button")[0]!;
  await fireEvent.contextMenu(row);
  expect(screen.getByRole("menu")).toBeTruthy();

  await shell.push(null);

  await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
});

function stubBox(element: HTMLElement, top: number) {
  element.getBoundingClientRect = () => ({ left: 90, top, right: 190, bottom: top + 30, width: 100, height: 30, x: 90, y: top, toJSON: () => ({}) });
}

test("a_scroll_moves_the_menu_with_its_item", async () => {
  const { origin } = harness();
  stubBox(origin, 90);
  await fireEvent.pointerDown(origin, { button: 2, clientX: 100, clientY: 100 });
  await fireEvent.contextMenu(origin, { clientX: 100, clientY: 100 });
  expect(menu().style.top).toBe("100px");

  stubBox(origin, 40);
  await fireEvent.scroll(document);

  expect(menu().style.top).toBe("50px");
  expect(menu().style.left).toBe("100px");
});

test("a_menu_closes_when_its_item_scrolls_out_of_view", async () => {
  const { origin } = harness();
  stubBox(origin, 90);
  await openByKeyboard(origin);

  stubBox(origin, -200);
  await fireEvent.scroll(document);

  expect(screen.queryByRole("menu")).toBeNull();
});
