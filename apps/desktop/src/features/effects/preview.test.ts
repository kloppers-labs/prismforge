import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers, wait } from "../../test/effects";
import { lifecycleFresh, lifecycleHidden } from "../../test/fixtures/lifecycles";
import { rainbowPreview } from "../../test/fixtures/previews";
import { demoConnected } from "../../test/fixtures/sessions";

afterEach(() => {
  setVisibility("visible");
  vi.unstubAllGlobals();
});

function setVisibility(state: DocumentVisibilityState) {
  Object.defineProperty(document, "visibilityState", { value: state, configurable: true });
  document.dispatchEvent(new Event("visibilitychange"));
}

function reduceMotion(reduce: boolean) {
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: reduce && query.includes("prefers-reduced-motion"),
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }));
}

async function animatedPreview() {
  const shell = fakeShell(demoConnected, effectHandlers({ preview_effect: () => rainbowPreview }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  const requests = () => shell.commands().filter((command) => command === "preview_effect").length;
  await vi.waitFor(() => expect(requests()).toBeGreaterThan(1), { timeout: 2_000 });
  return { shell, requests };
}

test("labels_the_preview_as_the_pattern_and_not_the_device", async () => {
  await animatedPreview();

  expect(screen.getByText(/Preview of the pattern on 12 LEDs\. It is not what the device shows\./)).toBeTruthy();
});

test("stops_requesting_while_the_document_is_hidden", async () => {
  const { requests } = await animatedPreview();

  setVisibility("hidden");
  await wait(50);
  const whileHidden = requests();
  await wait(300);
  expect(requests()).toBe(whileHidden);

  setVisibility("visible");
  await vi.waitFor(() => expect(requests()).toBeGreaterThan(whileHidden), { timeout: 2_000 });
});

test("stops_requesting_while_the_native_window_is_hidden_even_if_the_document_says_visible", async () => {
  const { shell, requests } = await animatedPreview();

  await shell.pushLifecycle({ ...lifecycleHidden, revision: 50 });
  await wait(50);
  const whileHidden = requests();
  await wait(300);
  expect(document.visibilityState).toBe("visible");
  expect(requests()).toBe(whileHidden);

  await shell.pushLifecycle({ ...lifecycleFresh, revision: 51 });
  await vi.waitFor(() => expect(requests()).toBeGreaterThan(whileHidden), { timeout: 2_000 });
});

test("stops_requesting_when_the_preview_panel_closes", async () => {
  const { requests } = await animatedPreview();

  await fireEvent.click(button("Hide preview"));
  await wait(50);
  const closed = requests();
  await wait(300);

  expect(requests()).toBe(closed);
  expect(screen.queryByText(/Preview of the pattern/)).toBeNull();
});

test("never_starts_or_stops_physical_output", async () => {
  const { shell } = await animatedPreview();
  await wait(300);

  expect(new Set(shell.commands())).toEqual(new Set(["view", "lifecycle", "profile_store", "groups", "effect_catalog", "preview_effect"]));
});

test("shows_one_still_frame_when_motion_is_reduced", async () => {
  reduceMotion(true);
  const shell = fakeShell(demoConnected, effectHandlers({ preview_effect: () => rainbowPreview }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  expect(await screen.findByText(/Motion is reduced, so the preview shows one still frame\./)).toBeTruthy();
  await wait(300);
  const previews = shell.calls.filter((call) => call.command === "preview_effect");
  expect(previews).toHaveLength(1);
  expect((previews[0]?.args.request as { frame_count: number }).frame_count).toBe(1);
});

test("requests_a_static_effect_once", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Gradient" }));

  await wait(400);

  expect(shell.commands().filter((command) => command === "preview_effect")).toHaveLength(1);
});
