import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import {
  demoConnected,
  openRgbAllRejected,
  openRgbConnecting,
  openRgbLostRetryFailed,
  openRgbLostWhileConnected,
  openRgbNoVersionAnswer,
  openRgbOldServer,
  openRgbRefused,
} from "../../test/fixtures/sessions";

test("shows_connection_failure_without_fake_devices", async () => {
  const shell = fakeShell(null, {
    connect_openrgb: () => shell.view(openRgbConnecting),
  });
  render(App);

  await fireEvent.click(await screen.findByRole("button", { name: "Connect" }));
  expect(shell.calls.find((call) => call.command === "connect_openrgb")?.args).toEqual({
    host: "127.0.0.1",
    port: 6742,
  });
  await screen.findByRole("heading", { name: "Connecting" });
  await shell.push(openRgbRefused);

  await screen.findByRole("heading", { name: "Waiting for OpenRGB" });
  expect(screen.getByText("Nothing answered at the OpenRGB server at 127.0.0.1:6742.")).toBeTruthy();
  expect(screen.queryByRole("navigation", { name: /Devices/ })).toBeNull();
  expect(screen.queryByRole("button", { name: /Identify/ })).toBeNull();
  expect(screen.queryByRole("complementary", { name: "Demo mode" })).toBeNull();
  expect(screen.queryByText(/Demo/)).toBeNull();
});

test("labels_demo_mode", async () => {
  fakeShell(demoConnected);
  render(App);

  const banner = await screen.findByRole("complementary", { name: "Demo mode" });
  expect(banner.textContent).toContain("These devices are simulated. Nothing is sent to hardware.");
  expect(screen.getByText("Simulated device. There is no hardware behind it.")).toBeTruthy();
  expect(screen.getByText("the demo backend")).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Diagnostics" }));
  expect(screen.getByRole("complementary", { name: "Demo mode" })).toBeTruthy();
  const diagnostics = screen.getByRole("complementary", { name: "Diagnostics" });
  expect(within(diagnostics).getByText("Simulated (demo mode)")).toBeTruthy();
});

test("shows_retry_attempt_and_cause", async () => {
  fakeShell(openRgbRefused);
  render(App);

  expect(await screen.findByText("Retrying, retry 3 next")).toBeTruthy();
  expect(screen.getByText("Nothing answered at the OpenRGB server at 127.0.0.1:6742.")).toBeTruthy();
  expect(screen.getByText(/PrismForge tries again on its own\. Retry 3 comes next/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "Stop retrying" })).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/Attempt \d+ failed/);
});

test("explains_a_lost_connection_without_inventing_failed_attempts", async () => {
  fakeShell(openRgbLostWhileConnected);
  render(App);

  const notice = await screen.findByRole("status", { name: "Connection status" });
  expect(notice.textContent).toContain("The connection was lost: the server closed the connection.");
  expect(notice.textContent).toContain("Retry 1 comes next");
  expect(screen.getByText("Retrying, retry 1 next")).toBeTruthy();
  const list = screen.getByRole("navigation", { name: /Devices/ });
  expect(within(list).getAllByText("Unavailable")).toHaveLength(2);
  expect(screen.getByText(/Identify is off\. Not connected/)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/Attempt \d+ failed|failed \d+ times/);
});

test("counts_retries_after_a_loss_as_the_core_does", async () => {
  fakeShell(openRgbLostRetryFailed);
  render(App);

  expect(await screen.findByText("Retrying, retry 2 next")).toBeTruthy();
  const notice = screen.getByRole("status", { name: "Connection status" });
  expect(notice.textContent).toContain("did not answer the version request");
  expect(notice.textContent).toContain("Retry 2 comes next");
});

test("explains_when_every_controller_was_rejected", async () => {
  fakeShell(openRgbAllRejected);
  render(App);

  expect(await screen.findByRole("heading", { name: "No usable devices" })).toBeTruthy();
  expect(screen.getByText("One controller sent data PrismForge could not read.")).toBeTruthy();
  expect(screen.getByText(/PrismForge could not read the data of the 1 controller OpenRGB reported\. Diagnostics lists it with the reason\./)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/reports no devices|other devices still work|Check the device list in OpenRGB/);
});

test("says_the_server_did_not_answer_the_version_request", async () => {
  fakeShell(openRgbNoVersionAnswer);
  render(App);

  const reason = await screen.findByText(/did not answer the version request/);
  expect(reason.textContent).toContain("It may still be starting, or it may have stopped responding.");
  expect(document.body.textContent).not.toMatch(/too old|protocol \d|update OpenRGB/i);
});

test("explains_an_old_server_on_the_connection_screen", async () => {
  fakeShell(openRgbOldServer);
  render(App);

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("uses SDK protocol 5. PrismForge needs protocol 6");
  expect(screen.getByRole("button", { name: "Connect" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Start demo mode" })).toBeTruthy();
});

test("ignores_a_view_older_than_the_one_shown", async () => {
  const shell = fakeShell(null);
  render(App);
  await screen.findByRole("button", { name: "Connect" });

  await shell.pushView({ revision: 10, session: demoConnected });
  await shell.pushView({ revision: 9, session: openRgbRefused });

  expect(await screen.findByRole("complementary", { name: "Demo mode" })).toBeTruthy();
  expect(screen.queryByText(/^Retrying/)).toBeNull();
});
