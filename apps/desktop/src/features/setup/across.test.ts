import { emit } from "@tauri-apps/api/event";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import App from "../../App.svelte";
import type { AcrossPreviewRequest } from "../../lib/api/bindings/AcrossPreviewRequest";
import type { LiveOutput } from "../../lib/api/bindings/LiveOutput";
import { LIVE_EVENT } from "../../lib/api/client";
import { fakeShell, refuse } from "../../test/backend";
import { acrossDemoSkipped, acrossDemoTower } from "../../test/fixtures/acrossReports";
import { previewDemoTower } from "../../test/fixtures/acrossPreviews";
import { liveRealAcross } from "../../test/fixtures/live";
import { demoConnected, openRgbAcrossRunning, demoAcrossRunning } from "../../test/fixtures/sessions";
import { setupRealTower } from "../../test/fixtures/setups";
import { componentRow, openSetup, setupHandlers } from "../../test/setup";

afterEach(() => {
  vi.unstubAllGlobals();
});

function reduceMotion() {
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: query.includes("prefers-reduced-motion"),
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }));
}

function fills(component: string): string[] {
  return [...document.querySelectorAll<SVGCircleElement>(`[data-component="${component}"] .led`)].map((led) => led.getAttribute("fill") ?? "");
}

function rgb(color: { r: number; g: number; b: number }): string {
  return `rgb(${color.r} ${color.g} ${color.b})`;
}

function panel() {
  return screen.getByRole("region", { name: "Effect across the setup" });
}

async function chooseEffect(label: string) {
  await fireEvent.change(within(panel()).getByRole("combobox", { name: "Effect" }), { target: { value: label } });
}

test("the_draft_preview_draws_the_pattern_across_the_unsaved_drawing_and_sends_nothing", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();

  await chooseEffect("rainbow-wave");

  await waitFor(() => expect(shell.commands()).toContain("preview_across"));
  const request = shell.calls.find((call) => call.command === "preview_across")!.args.request as AcrossPreviewRequest;
  expect(request.view).toBe("v-pc");
  expect(request.flow).toEqual({ Along: { angle: 0 } });
  expect(request.instance.effect).toBe("rainbow-wave");
  const stage = screen.getByRole("group", { name: "Colours in the drawing" });
  expect(stage.textContent).toContain("Draft");
  expect(stage.textContent).toContain("The pattern only. PrismForge sends nothing for it.");
  const graphics = previewDemoTower.targets.find((target) => target.target.device === "demo-graphics-card")!.frames[0]!;
  await waitFor(() => expect(fills("c-graphics")).toEqual(graphics.map(rgb)));
  expect(shell.commands()).not.toContain("apply_across");
  expect(shell.commands()).not.toContain("apply_effect_batch");
});

test("mirrored_fans_show_the_same_colours_and_chain_parts_show_their_own_part", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();

  await chooseEffect("rainbow-wave");

  const hub = previewDemoTower.targets.find((target) => target.target.zone === 1)!.frames[0]!;
  const cooler = previewDemoTower.targets.find((target) => target.target.zone === 2)!.frames[0]!;
  await waitFor(() => expect(fills("c-front-fan-1")).toEqual(hub.slice(0, 8).map(rgb)));
  expect(fills("c-rear-fan")).toEqual(fills("c-front-fan-1"));
  expect(fills("c-cooler-left")).toEqual(cooler.slice(0, 8).map(rgb));
  expect(fills("c-cooler-right")).toEqual(cooler.slice(8, 16).map(rgb));
});

test("reduced_motion_keeps_the_draft_at_one_still_frame", async () => {
  reduceMotion();
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();

  await chooseEffect("rainbow-wave");

  await waitFor(() => expect(shell.commands()).toContain("preview_across"));
  expect((shell.calls.find((call) => call.command === "preview_across")!.args.request as AcrossPreviewRequest).frame_count).toBe(1);
  expect(await screen.findByText(/Motion is reduced, so the drawing shows one still frame\./)).toBeTruthy();
});

test("the_live_output_is_drawn_on_the_components_only_while_the_setup_is_shown", async () => {
  const shell = fakeShell(openRgbAcrossRunning, setupHandlers(setupRealTower));
  render(App);
  await openSetup();
  const watched = () => shell.calls.filter((call) => call.command === "watch_live_output").map((call) => call.args.targets);

  await fireEvent.click(screen.getByRole("button", { name: "Live output" }));
  await waitFor(() => expect(watched().at(-1)).toHaveLength(3));
  await emit(LIVE_EVENT, liveRealAcross);

  const stage = screen.getByRole("group", { name: "Colours in the drawing" });
  expect(stage.textContent).toContain("Live");
  expect(stage.textContent).toContain("What PrismForge sends now. PrismForge cannot see the device, so this is not a reading of the lights.");
  const zone = liveRealAcross.devices[0]!.zones[0]!.colors;
  await waitFor(() => expect(fills("c-front-fan-2")).toEqual(zone.slice(0, 8).map(rgb)));
  await fireEvent.click(within(screen.getByRole("navigation", { name: "Main" })).getByRole("button", { name: "Devices" }));
  await waitFor(() => expect(watched().at(-1)).toEqual([]));
});

test("a_flashing_live_output_is_shown_slowed_in_the_drawing", async () => {
  fakeShell(openRgbAcrossRunning, setupHandlers(setupRealTower));
  render(App);
  await openSetup();
  await fireEvent.click(screen.getByRole("button", { name: "Live output" }));
  const strobe = (on: boolean): LiveOutput => ({
    devices: liveRealAcross.devices.map((device) => ({
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

test("apply_across_sends_the_saved_view_the_flow_and_the_effect_and_shows_each_target_and_the_dark_leds", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { apply_across: () => acrossDemoTower, preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();
  await chooseEffect("rainbow-wave");
  await fireEvent.change(within(panel()).getByRole("spinbutton", { name: "Angle" }), { target: { value: "90" } });

  await fireEvent.click(within(panel()).getByRole("button", { name: "Apply across the PC view" }));

  await waitFor(() => expect(shell.commands()).toContain("apply_across"));
  const args = shell.calls.find((call) => call.command === "apply_across")!.args;
  expect(args.view).toBe("v-pc");
  expect(args.flow).toEqual({ Along: { angle: 90 } });
  expect((args.effect as { effect: string }).effect).toBe("rainbow-wave");
  expect(args.targets).toBeNull();
  const results = await within(panel()).findByRole("status", { name: "Results for each target" });
  expect(results.textContent).toContain("Rainbow wave across the PC view: 6 of 6 targets accepted.");
  expect(results.textContent).toContain("Addressable Header 1 on Demo Motherboard: 22 of 30 LEDs have no place and are sent black.");
});

test("outputs_whose_zone_changed_are_listed_with_the_reason_and_retry_resends_only_what_can_take_it", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(undefined, { apply_across: () => acrossDemoSkipped, preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();
  await chooseEffect("rainbow-wave");
  await fireEvent.click(within(panel()).getByRole("button", { name: "Apply across the PC view" }));

  const results = await within(panel()).findByRole("status", { name: "Results for each target" });
  expect(results.textContent).toContain("Board logo on Demo Motherboard (Board edge): not sent. This zone is no longer Board logo, so its parts may not fit. Place it again on the zone it is on now.");
  expect(results.textContent).toMatch(/Demo Light Bar: skipped\. This device has no Direct Mode\./);
  await fireEvent.click(within(results).getByRole("button", { name: "Retry 1 target" }));

  await waitFor(() => expect(shell.commands().filter((command) => command === "apply_across")).toHaveLength(2));
  const retry = shell.calls.filter((call) => call.command === "apply_across")[1]!.args;
  expect(retry.targets).toEqual([{ device: "demo-graphics-card", zone: null }]);
  expect(retry.view).toBe("v-pc");
});

test("apply_across_waits_for_a_saved_setup", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { preview_across: () => previewDemoTower }));
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Graphics card"));
  await fireEvent.keyDown(componentRow(list, "Graphics card"), { key: "]" });
  await chooseEffect("rainbow-wave");

  const apply = within(panel()).getByRole("button", { name: "Apply across the PC view" }) as HTMLButtonElement;

  expect(apply.disabled).toBe(true);
  expect(panel().textContent).toContain("Save the setup first. Effects run across the saved setup, and the draft shows your unsaved drawing.");
});

test("the_flow_is_set_by_an_angle_or_a_centre_and_shown_with_the_mode", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();
  await chooseEffect("rainbow-wave");

  await fireEvent.click(within(panel()).getByRole("radio", { name: "Out from a centre" }));
  await fireEvent.change(within(panel()).getByRole("spinbutton", { name: "Centre across" }), { target: { value: "25" } });

  expect(panel().textContent).toContain("Across the setup: PC view, out from a centre 25% across and 50% down");
});

test("stop_stops_the_targets_that_run_across_this_view", async () => {
  const shell = fakeShell(demoAcrossRunning, setupHandlers(undefined, { stop_effect_batch: () => ({ results: [] }) }));
  render(App);
  await openSetup();

  await fireEvent.click(within(panel()).getByRole("button", { name: "Stop across the PC view" }));

  await waitFor(() => expect(shell.commands()).toContain("stop_effect_batch"));
  const targets = shell.calls.find((call) => call.command === "stop_effect_batch")!.args.targets as unknown[];
  expect(targets).toHaveLength(6);
});

test("a_refused_apply_says_why", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { apply_across: refuse("ProfileApplying"), preview_across: () => previewDemoTower }));
  render(App);
  await openSetup();
  await chooseEffect("rainbow-wave");

  await fireEvent.click(within(panel()).getByRole("button", { name: "Apply across the PC view" }));

  expect((await within(panel()).findByRole("alert")).textContent).toContain("PrismForge is applying a profile.");
});
