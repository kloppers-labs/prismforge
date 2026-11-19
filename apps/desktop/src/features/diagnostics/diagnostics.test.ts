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
  await screen.findByText("Retrying");

  const drawer = await openDiagnostics();

  expect(within(drawer).getByText("Unreachable")).toBeTruthy();
  expect(within(drawer).getByText('{"Unreachable":{"detail":"connection refused"}}')).toBeTruthy();
  const counters = within(drawer).getByRole("heading", { name: "Counters" }).parentElement as HTMLElement;
  expect(within(counters).getByText("Failed connects").nextElementSibling?.textContent).toBe("3");
  expect(within(drawer).getAllByText(/RetryScheduled retry \d+ in \d+ ms/).length).toBeGreaterThan(0);

  await fireEvent.keyDown(drawer, { key: "Escape" });
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

test("diagnostics_copy_as_text_for_a_bug_report", async () => {
  const copied: string[] = [];
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: (text: string) => (copied.push(text), Promise.resolve()) } });
  fakeShell(openRgbRefused);
  render(App);
  await screen.findByText("Retrying");
  const drawer = await openDiagnostics();

  await fireEvent.click(within(drawer).getByRole("button", { name: "Copy as text" }));

  expect(await within(drawer).findByText("Copied. Paste it into your report.")).toBeTruthy();
  expect(copied).toHaveLength(1);
  expect(copied[0]).toMatch(/^PrismForge diagnostics/);
  expect(copied[0]).toMatch(/Failed connects: 3/);
  expect(copied[0]).toMatch(/Unreachable/);
});

test("diagnostics_name_devices_the_way_the_user_does_beside_their_references", async () => {
  fakeShell(demoEffectsRunning);
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  const drawer = await openDiagnostics();

  const references = within(drawer).getByRole("region", { name: "Device references" });
  expect(within(references).getByText("Demo Graphics Card")).toBeTruthy();
  expect(within(references).getByText("demo-graphics-card")).toBeTruthy();
  const output = within(drawer).getByRole("region", { name: "Output" });
  expect(within(output).getAllByText("Demo Graphics Card").length).toBeGreaterThan(0);
});

test("the_copied_diagnostics_leave_out_the_folder_path_and_say_what_they_hold", async () => {
  const copied: string[] = [];
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: (text: string) => (copied.push(text), Promise.resolve()) } });
  fakeShell(demoEffectsRunning);
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  const drawer = await openDiagnostics();
  await within(drawer).findByText(/store-folder\/demo-store/);

  await fireEvent.click(within(drawer).getByRole("button", { name: "Copy as text" }));

  await within(drawer).findByText("Copied. Paste it into your report.");
  expect(copied[0]).not.toMatch(/store-folder/);
  expect(within(drawer).getByText(/The copy leaves out the folder path\./)).toBeTruthy();
});
