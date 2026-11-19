import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { effectHandlers } from "../../test/effects";
import { demoConnected } from "../../test/fixtures/sessions";
import { storeTwo } from "../../test/fixtures/stores";
import { connectionChip, waitForConnection } from "../../test/shell";
import { guardNativeMenu, nativeMenu } from "./nativeMenu";

let release: (() => void) | null = null;

afterEach(() => {
  release?.();
  release = null;
  document.body.replaceChildren();
});

function place(html: string): HTMLElement {
  const holder = document.createElement("div");
  holder.innerHTML = html;
  document.body.append(holder);
  return holder.firstElementChild as HTMLElement;
}

function rightClick(target: Element): boolean {
  return target.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
}

test("production_keeps_the_webview_menu_only_in_text_fields", () => {
  release = guardNativeMenu(window, "production");

  expect(rightClick(place("<p>Devices</p>"))).toBe(false);
  expect(rightClick(place("<button>Rescan</button>"))).toBe(false);
  expect(rightClick(place("<input type='checkbox'>"))).toBe(false);
  expect(rightClick(place("<input type='radio'>"))).toBe(false);
  expect(rightClick(place("<input type='text'>"))).toBe(true);
  expect(rightClick(place("<input>"))).toBe(true);
  expect(rightClick(place("<input type='number'>"))).toBe(true);
  expect(rightClick(place("<textarea></textarea>"))).toBe(true);
  expect(rightClick(place("<div contenteditable='true'>note</div>"))).toBe(true);
});

test("development_keeps_the_webview_menu_where_there_is_no_app_menu", () => {
  release = guardNativeMenu(window, "development");

  expect(rightClick(place("<p>Devices</p>"))).toBe(true);
  expect(rightClick(place("<input type='checkbox'>"))).toBe(true);
  expect(rightClick(place("<textarea></textarea>"))).toBe(true);
});

test("the_decision_differs_between_production_and_development", () => {
  const text = place("<p>Profiles</p>");
  const field = place("<input type='search'>");

  expect(nativeMenu(text, "production")).toBe("suppress");
  expect(nativeMenu(text, "development")).toBe("allow");
  expect(nativeMenu(field, "production")).toBe("allow");
  expect(nativeMenu(field, "development")).toBe("allow");
  expect(nativeMenu(null, "production")).toBe("suppress");
});

test("an_app_menu_replaces_the_webview_menu_in_development_too", async () => {
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  release = guardNativeMenu(window, "development");

  const shown = await fireEvent.contextMenu(connectionChip());

  expect(shown).toBe(false);
  expect(screen.getByRole("menu", { name: "Connection actions" })).toBeTruthy();
  expect(await fireEvent.contextMenu(screen.getByRole("main"))).toBe(true);
});

test("production_suppresses_the_webview_menu_on_disabled_controls_and_where_a_handler_stops_the_event", () => {
  release = guardNativeMenu(window, "production");
  const disabled = place("<button disabled>Identify device</button>");
  const sealed = place("<div><span>Busy</span></div>");
  sealed.addEventListener("contextmenu", (event) => event.stopPropagation());

  expect(rightClick(disabled)).toBe(false);
  expect(rightClick(sealed.querySelector("span")!)).toBe(false);
});

test("an_app_menu_still_opens_in_production", async () => {
  fakeShell(demoConnected, { ...effectHandlers(), profile_store: () => storeTwo });
  render(App);
  await waitForConnection("Connected");
  release = guardNativeMenu(window, "production");

  const shown = await fireEvent.contextMenu(connectionChip());

  expect(shown).toBe(false);
  expect(screen.getByRole("menu", { name: "Connection actions" })).toBeTruthy();
});
