import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { demoEffectsRunning, openRgbDegraded, openRgbRefused } from "../../test/fixtures/sessions";

async function openDiagnostics() {
  await fireEvent.click(await screen.findByRole("button", { name: "Diagnostics" }));
  return screen.getByRole("complementary", { name: "Diagnostics" });
}

test("keeps_runtime_details_in_diagnostics", async () => {
  fakeShell(openRgbDegraded);
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  expect(document.body.textContent).not.toMatch(/Controller ID|PrismForge fake OpenRGB|dev-[0-9a-f]{16}/);

  const drawer = await openDiagnostics();

  expect(within(drawer).getByText("127.0.0.1:6742")).toBeTruthy();
  expect(within(drawer).getByText("6")).toBeTruthy();
  expect(within(drawer).getByText("PrismForge fake OpenRGB")).toBeTruthy();
  expect(within(drawer).getByText(/Controller ID 5: InconsistentData/)).toBeTruthy();
  expect(within(drawer).getAllByText(/^dev-[0-9a-f]{16}$/).length).toBeGreaterThan(0);
});

test("shows_the_last_error_and_real_counters", async () => {
  fakeShell(openRgbRefused);
  render(App);
  await screen.findByText("Retrying, retry 3 next");

  const drawer = await openDiagnostics();

  expect(within(drawer).getByText("Unreachable")).toBeTruthy();
  expect(within(drawer).getByText('{"Unreachable":{"detail":"connection refused"}}')).toBeTruthy();
  const counters = within(drawer).getByRole("heading", { name: "Counters" }).parentElement as HTMLElement;
  expect(within(counters).getByText("Failed connects").nextElementSibling?.textContent).toBe("3");
  expect(within(drawer).getAllByText(/RetryScheduled retry \d+ in \d+ ms/).length).toBeGreaterThan(0);

  await fireEvent.keyDown(window, { key: "Escape" });
  expect(screen.queryByRole("complementary", { name: "Diagnostics" })).toBeNull();
});

test("shows_output_counters_and_owners_in_diagnostics_only", async () => {
  fakeShell(demoEffectsRunning);
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  expect(document.body.textContent).not.toMatch(/Frames written|Ticks skipped/);

  const drawer = await openDiagnostics();

  const counters = within(drawer).getByRole("heading", { name: "Counters" }).parentElement as HTMLElement;
  const counter = (label: string) => within(counters).getByText(label).nextElementSibling?.textContent;
  const real = demoEffectsRunning.diagnostics.counters;
  expect(counter("Frames written")).toBe(String(real.frames_written));
  expect(counter("Ticks skipped")).toBe(String(real.ticks_skipped));
  expect(counter("Writes refused")).toBe(String(real.output_writes_refused));
  expect(counter("Writes that ended the connection")).toBe(String(real.output_writes_failed));
  expect(Number(counter("Frames written"))).toBeGreaterThan(0);
  expect(within(drawer).getByText("30 per second")).toBeTruthy();
  expect(within(drawer).getByText(/"Refused":\{"error":\{"Rejected"/)).toBeTruthy();
});
