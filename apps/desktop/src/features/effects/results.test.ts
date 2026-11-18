import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers, wait, withEffect, withOwner } from "../../test/effects";
import { demoConnected, demoEffectsRunning } from "../../test/fixtures/sessions";

const PRESENT_EFFECT_CLAIM = /now runs|PrismForge runs [A-Z]/;
const GPU = "demo-graphics-card";
const rainbow = { effect: "rainbow-wave", values: {} };

async function applyRainbowToGraphicsCard(shell: ReturnType<typeof fakeShell>) {
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  await fireEvent.click(button("Apply effect"));
  await screen.findByText(/^PrismForge accepted Rainbow wave for Demo Graphics Card\./);
  await shell.push(withEffect(demoConnected, { device: GPU, zone: null }, rainbow, "Running"));
  await wait(50);
}

test("no_stale_effect_claim_after_a_hardware_mode_or_a_declaration", async () => {
  const shell = fakeShell(
    demoConnected,
    effectHandlers({ apply_effect: () => null, select_mode: () => null, declare_external: () => null }),
  );
  render(App);
  await applyRainbowToGraphicsCard(shell);

  await fireEvent.click(button("Use Breathing"));
  await screen.findByText(/^The demo backend accepted the request to use hardware mode Breathing\./);
  await shell.push(withOwner(demoConnected, GPU, { HardwareMode: { mode: 2 } }));
  await wait(50);
  expect(screen.getByText(/^Hardware mode Breathing, selected in PrismForge\./)).toBeTruthy();
  expect(screen.queryByText(/accepted Rainbow wave/)).toBeNull();
  expect(document.body.textContent).not.toMatch(PRESENT_EFFECT_CLAIM);

  await fireEvent.click(button("Another program controls this device"));
  await shell.push(withOwner(demoConnected, GPU, { External: { label: "another program" } }));
  await wait(50);
  expect(screen.getByText(/^You said another program controls this device\. PrismForge sends nothing to it\.$/)).toBeTruthy();
  expect(screen.queryByText(/accepted Rainbow wave|accepted the request to use hardware mode/)).toBeNull();
  expect(document.body.textContent).not.toMatch(PRESENT_EFFECT_CLAIM);
});

test("a_refused_device_does_not_read_as_running", async () => {
  fakeShell(demoEffectsRunning, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Memory Module");

  await fireEvent.click(button("Apply effect"));

  expect(await screen.findByText(/^PrismForge accepted Solid colour for Demo Memory Module\./)).toBeTruthy();
  expect(screen.getByText(/^Refused\./)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(PRESENT_EFFECT_CLAIM);
});

test("a_paused_device_does_not_read_as_running", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(button("Apply effect"));
  await screen.findByText(/^PrismForge accepted Rainbow wave for Demo Graphics Card\./);

  await shell.push(withEffect(demoEffectsRunning, { device: GPU, zone: null }, rainbow, { Paused: { reason: "InventoryStale" } }));
  await wait(50);

  expect(screen.getByText(/^Paused\. The device list is being read again\./)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(PRESENT_EFFECT_CLAIM);
});

test("the_effect_result_goes_when_a_newer_view_no_longer_has_the_effect", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await applyRainbowToGraphicsCard(shell);
  expect(screen.getByText(/^PrismForge accepted Rainbow wave/)).toBeTruthy();

  await shell.push(demoConnected);
  await wait(50);

  expect(screen.queryByText(/^PrismForge accepted Rainbow wave/)).toBeNull();
});

test("the_mode_result_goes_after_a_later_apply", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ apply_effect: () => null, select_mode: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(button("Use Breathing"));
  await screen.findByText(/^The demo backend accepted the request to use hardware mode Breathing\./);

  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  await fireEvent.click(button("Apply effect"));
  await screen.findByText(/^PrismForge accepted Rainbow wave/);

  expect(screen.queryByText(/accepted the request to use hardware mode/)).toBeNull();
  expect(shell.commands()).toContain("apply_effect");
});

test("the_frame_rate_in_use_comes_from_the_snapshot", async () => {
  const shell = fakeShell(demoConnected, effectHandlers({ set_frame_rate: () => 60 }));
  render(App);
  await chooseTarget("Demo Graphics Card");

  await fireEvent.input(screen.getByRole("spinbutton", { name: "Frames per second" }), { target: { value: "75" } });
  await fireEvent.click(button("Set frame rate"));

  expect(await screen.findByText(/^You asked for 75 frames per second\. PrismForge set the nearest allowed rate, 60\./)).toBeTruthy();
  expect(screen.getByText(/Now 30\./)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/PrismForge uses \d+ frames per second/);

  const later = structuredClone(demoConnected);
  later.output.frame_rate = 24;
  await shell.push(later);
  await wait(50);

  expect(screen.getByText(/Now 24\./)).toBeTruthy();
  expect(screen.queryByText(/You asked for 75/)).toBeNull();
});
