import { emit } from "@tauri-apps/api/event";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { LIVE_EVENT } from "../../lib/api/client";
import type { LiveOutput } from "../../lib/api/bindings/LiveOutput";
import { fakeShell, type Call } from "../../test/backend";
import { button, chooseTarget, effectHandlers, openEffects } from "../../test/effects";
import { lifecycleFresh, lifecycleHidden } from "../../test/fixtures/lifecycles";
import { liveKeyboardAndStrip } from "../../test/fixtures/live";
import { rainbowPreview } from "../../test/fixtures/previews";
import { demoConnected, openRgbKeyboardRunning } from "../../test/fixtures/sessions";

afterEach(() => {
  vi.unstubAllGlobals();
});

function watched(calls: Call[]) {
  return calls.filter((call) => call.command === "watch_live_output").map((call) => call.args.targets);
}

function writes(calls: Call[]): string[] {
  const reads = ["view", "lifecycle", "groups", "effect_catalog", "preview_effect", "profile_store", "watch_live_output"];
  return calls.map((call) => call.command).filter((command) => !reads.includes(command));
}

async function keyboardAndStrip() {
  const shell = fakeShell(openRgbKeyboardRunning, effectHandlers({ preview_effect: () => rainbowPreview }));
  render(App);
  await openEffects();
  await fireEvent.click(screen.getByRole("button", { name: "Select all" }));
  await screen.findByRole("heading", { level: 2, name: "2 targets" });
  return shell;
}

test("the_draft_preview_and_the_live_output_are_two_states_each_named_and_marked", async () => {
  const shell = await keyboardAndStrip();

  const draft = button("Draft preview");
  expect(draft.getAttribute("aria-pressed")).toBe("true");
  expect(screen.getByText(/The pattern only\./).closest("p")?.textContent).toMatch(/◇\s*Draft The pattern only\. PrismForge sends nothing for it\./);
  expect(screen.getByText(/^Preview of the pattern on each target's LEDs, on one clock/)).toBeTruthy();
  expect(watched(shell.calls)).toEqual([]);

  await fireEvent.click(button("Live output"));

  expect(button("Live output").getAttribute("aria-pressed")).toBe("true");
  expect(screen.getByText(/What PrismForge sends now\./).closest("p")?.textContent).toMatch(/●\s*Live What PrismForge sends now\. PrismForge cannot see the device/);
  expect(screen.queryByText(/^Preview of the pattern/)).toBeNull();
  await waitFor(() => expect(watched(shell.calls)).toHaveLength(1));
});

test("the_live_output_draws_each_zone_in_its_shape_from_the_frames_rust_sends", async () => {
  const shell = await keyboardAndStrip();
  await fireEvent.click(button("Live output"));
  await waitFor(() => expect(watched(shell.calls)).toHaveLength(1));
  const [keys, desk] = liveKeyboardAndStrip.devices;
  expect(watched(shell.calls)[0]).toEqual([
    { device: keys!.device, zone: null },
    { device: desk!.device, zone: null },
  ]);

  await emit(LIVE_EVENT, liveKeyboardAndStrip);

  const grid = await screen.findByRole("img", { name: "Keys: a grid of 12 by 4, as PrismForge last sent them" });
  expect(grid.children).toHaveLength(48);
  const first = keys!.zones[0]!.colors[0]!;
  expect((grid.children[0] as HTMLElement).style.getPropertyValue("--led")).toBe(`rgb(${first.r} ${first.g} ${first.b})`);
  expect(screen.getByRole("img", { name: "Header: a row of 12 LEDs, as PrismForge last sent them" }).children).toHaveLength(12);
  expect(writes(shell.calls)).toEqual([]);
});

test("the_live_output_is_asked_for_only_while_it_is_on_screen", async () => {
  const shell = await keyboardAndStrip();
  await fireEvent.click(button("Live output"));
  await waitFor(() => expect(watched(shell.calls)).toHaveLength(1));

  await shell.pushLifecycle({ ...lifecycleHidden, revision: 50 });
  await waitFor(() => expect(watched(shell.calls).at(-1)).toEqual([]));
  await shell.pushLifecycle({ ...lifecycleFresh, revision: 51 });
  await waitFor(() => expect(watched(shell.calls).at(-1)).toHaveLength(2));

  await fireEvent.click(button("Hide preview"));
  await waitFor(() => expect(watched(shell.calls).at(-1)).toEqual([]));
  await fireEvent.click(button("Show preview"));
  await waitFor(() => expect(watched(shell.calls).at(-1)).toHaveLength(2));

  await fireEvent.click(button("Draft preview"));
  await waitFor(() => expect(watched(shell.calls).at(-1)).toEqual([]));
  expect(writes(shell.calls)).toEqual([]);
});

test("a_flashing_live_output_is_shown_slowed_on_the_screen", async () => {
  const shell = await keyboardAndStrip();
  await fireEvent.click(button("Live output"));
  await waitFor(() => expect(watched(shell.calls)).toHaveLength(1));
  const strobe = (on: boolean): LiveOutput => ({
    devices: liveKeyboardAndStrip.devices.map((device) => ({
      ...device,
      zones: device.zones.map((zone) => ({ ...zone, colors: zone.colors.map(() => (on ? { r: 255, g: 255, b: 255 } : { r: 0, g: 0, b: 0 })) })),
    })),
  });
  let now = 0;
  vi.spyOn(performance, "now").mockImplementation(() => now);

  for (let index = 0; index < 20; index += 1) {
    now = index * 40;
    await emit(LIVE_EVENT, strobe(index % 2 === 0));
  }

  expect(await screen.findByText("Fast flashing is shown slowed on the screen. The devices get every frame.")).toBeTruthy();
});

test("the_live_view_says_when_nothing_runs_on_the_targets", async () => {
  fakeShell(demoConnected, effectHandlers({ preview_effect: () => rainbowPreview }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await fireEvent.click(button("Live output"));

  expect(screen.getByText("No PrismForge effect runs on this target, so nothing is sent.")).toBeTruthy();
  expect(screen.getByText(/to the demo backend now\. Nothing reaches hardware\./)).toBeTruthy();
});

test("a_flashing_draft_preview_is_shown_slowed_on_the_screen", async () => {
  const white = { r: 255, g: 255, b: 255 };
  const black = { r: 0, g: 0, b: 0 };
  const strobe = (args: Record<string, unknown>) => {
    const request = args.request as { led_count: number; start_ms: number; frame_count: number; interval_ms: number };
    return {
      led_count: request.led_count,
      start_ms: request.start_ms,
      interval_ms: request.interval_ms,
      is_static: false,
      frames: Array.from({ length: request.frame_count }, (_, index) => Array(request.led_count).fill(index % 2 === 0 ? white : black)),
    };
  };
  fakeShell(demoConnected, effectHandlers({ preview_effect: strobe }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  expect(await screen.findByText("Fast flashing is shown slowed on the screen. The devices get every frame.", {}, { timeout: 2_000 })).toBeTruthy();
});

test("the_draft_preview_draws_a_row_for_every_target_at_its_own_led_count", async () => {
  const sized = (args: Record<string, unknown>) => {
    const request = args.request as { led_count: number; start_ms: number; frame_count: number; interval_ms: number };
    return {
      led_count: request.led_count,
      start_ms: request.start_ms,
      interval_ms: request.interval_ms,
      is_static: true,
      frames: [Array.from({ length: request.led_count }, (_, index) => ({ r: index * 8, g: 64, b: 128 }))],
    };
  };
  fakeShell(demoConnected, effectHandlers({ preview_effect: sized }));
  render(App);
  await openEffects();
  await fireEvent.click(screen.getByRole("button", { name: "Select all" }));
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  await waitFor(() => expect(document.querySelectorAll(".stage .led:not(.off)")).toHaveLength(8 + 8 + 52 + 12));
  expect(document.querySelectorAll(".stage .led.off")).toHaveLength(0);
});
