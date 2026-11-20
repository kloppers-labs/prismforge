import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { lifecycleAsking } from "../../test/fixtures/lifecycles";
import { demoConnected, openRgbDegraded } from "../../test/fixtures/sessions";
import { storeEmpty } from "../../test/fixtures/stores";
import { viewApplying, viewConnectApplied } from "../../test/fixtures/views";
import { connectionChip, current, findStrip, go, notices, openConnection, strip, waitForConnection } from "../../test/shell";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function narrowLayout() {
  vi.stubGlobal("matchMedia", (query: string) => ({ matches: query.includes("max-width"), media: query, addEventListener() {}, removeEventListener() {} }));
}

function inert(element: Element): boolean {
  for (let node: Element | null = element; node; node = node.parentElement) {
    if ((node as HTMLElement).inert) {
      return true;
    }
  }
  return false;
}

async function covered(handlers: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
  narrowLayout();
  const shell = fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeEmpty, ...handlers });
  render(App);
  await waitForConnection("Connected");
  await go("Diagnostics");
  expect(screen.getByRole("main").inert).toBe(true);
  return shell;
}

test("skip_to_content_moves_focus_to_the_page_when_diagnostics_covers_it", async () => {
  await covered();
  const main = screen.getByRole("main");
  const inertWhenFocused: boolean[] = [];
  const focus = HTMLElement.prototype.focus;
  vi.spyOn(HTMLElement.prototype, "focus").mockImplementation(function (this: HTMLElement, options?: FocusOptions) {
    if (this === main) {
      inertWhenFocused.push(main.inert);
    }
    focus.call(this, options);
  });

  await fireEvent.click(screen.getByRole("button", { name: "Skip to content" }));

  await waitFor(() => expect(document.activeElement).toBe(main));
  expect(inertWhenFocused.at(-1)).toBe(false);
  expect(screen.queryByRole("complementary", { name: "Diagnostics" })).toBeNull();
});

test("f6_stays_inside_an_open_dialog", async () => {
  fakeShell(demoConnected, { lifecycle: () => lifecycleAsking });
  render(App);
  const dialog = await screen.findByRole("dialog");
  await waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));

  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "F6" });

  expect(dialog.contains(document.activeElement)).toBe(true);
});

test("a_notice_that_arrives_while_diagnostics_covers_the_page_is_shown_and_announced", async () => {
  const shell = await covered();

  await shell.pushView({ ...viewConnectApplied, session: openRgbDegraded, revision: 50 });

  const slot = await waitFor(() => notices() ?? Promise.reject(new Error("no notices")));
  expect(inert(slot)).toBe(false);
  expect(inert(screen.getByRole("status", { name: "New notices" }))).toBe(false);
  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();
});

test("a_notice_that_comes_back_with_the_same_text_is_announced_again", async () => {
  const shell = fakeShell(openRgbDegraded, effectHandlers());
  render(App);
  await screen.findByRole("status", { name: "Connection status" });
  await shell.pushView({ ...viewConnectApplied, session: openRgbDegraded, revision: 50 });
  const announcer = await screen.findByRole("status", { name: "New notices" });
  await waitFor(() => expect(announcer.textContent).toMatch(/New: PrismForge applied Evening/));
  const first = announcer.firstElementChild;

  await shell.pushView({ ...viewConnectApplied, session: openRgbDegraded, connect_apply: null, revision: 51 });
  await waitFor(() => expect(within(notices()!).queryByRole("button", { name: /Show all/ })).toBeNull());
  await shell.pushView({ ...viewConnectApplied, session: openRgbDegraded, revision: 52 });

  await waitFor(() => expect(announcer.firstElementChild).not.toBe(first));
  expect(announcer.textContent).toMatch(/New: PrismForge applied Evening/);
});

test("the_settings_shortcut_follows_the_comma_character_not_its_key_position", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await findStrip();

  await fireEvent.keyDown(document.body, { key: "w", code: "Comma", ctrlKey: true });
  expect(current()).toEqual(["Devices"]);
  await fireEvent.keyDown(document.body, { key: ",", code: "KeyW", ctrlKey: true });
  expect(current()).toEqual(["Settings"]);
});

test("a_status_panel_keeps_its_place_when_the_window_is_resized", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await waitForConnection("Connected");
  const panel = await openConnection();
  const before = panel.style.maxHeight;

  vi.stubGlobal("innerHeight", 300);
  window.dispatchEvent(new Event("resize"));

  await waitFor(() => expect(panel.style.maxHeight).not.toBe(before));
  expect(panel.style.maxHeight).toBe("284px");
});

test("cancelling_the_first_close_question_after_it_closed_diagnostics_keeps_focus_on_the_page", async () => {
  const shell = await covered({ answer_close: () => null });
  within(screen.getByRole("complementary", { name: "Diagnostics" })).getByRole("button", { name: "Close" }).focus();

  await shell.pushLifecycle({ ...lifecycleAsking, revision: 40 });
  const dialog = await screen.findByRole("dialog");
  await fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  await shell.pushLifecycle({ ...lifecycleAsking, close_prompt: false, revision: 41 });

  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("main"));
});

test("escape_in_the_page_leaves_docked_diagnostics_open", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await waitForConnection("Connected");
  await go("Diagnostics");
  const page = within(screen.getByRole("main")).getAllByRole("button")[0] as HTMLElement;

  await fireEvent.keyDown(page, { key: "Escape" });
  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();

  await fireEvent.keyDown(within(screen.getByRole("complementary", { name: "Diagnostics" })).getByRole("button", { name: "Close" }), { key: "Escape" });
  expect(screen.queryByRole("complementary", { name: "Diagnostics" })).toBeNull();
});

test("the_strip_shows_no_profile_chip_before_any_connection", async () => {
  fakeShell(null);
  render(App);
  await findStrip();
  await waitFor(() => expect(connectionChip()).toBeTruthy());

  expect(within(strip()).queryByRole("button", { name: /^Profile:/ })).toBeNull();
});

test("an_automatic_apply_keeps_on_connect_in_the_profile_chip_name", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await findStrip();

  await shell.pushView({ ...viewApplying, applying: { ...viewApplying.applying!, automatic: true }, revision: 50 });

  expect(await within(strip()).findByRole("button", { name: "Profile: applying Evening by a rule" })).toBeTruthy();
});
