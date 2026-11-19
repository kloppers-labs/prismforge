import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import type { LifecycleView } from "../../lib/api/bindings/LifecycleView";
import { lifecycleStartPath } from "../../test/fixtures/lifecycles";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { summarise } from "./wording";
import { connectionChip, findDemoMarker, openConnection, waitForConnection } from "../../test/shell";
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
  expect(document.querySelector("[data-demo]")).toBeNull();
  expect(screen.queryByText(/Demo/)).toBeNull();
});

test("labels_demo_mode", async () => {
  fakeShell(demoConnected);
  render(App);

  const marker = await findDemoMarker();
  expect(within(marker).getByText("simulated devices")).toBeTruthy();
  const panel = await openConnection();
  expect(within(panel).getByText("Demo mode: these devices are simulated. Nothing is sent to hardware.")).toBeTruthy();
  expect(screen.getByText("Simulated device. There is no hardware behind it.")).toBeTruthy();
  expect(within(connectionChip()).getByText("Demo backend")).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Diagnostics" }));
  expect(await findDemoMarker()).toBeTruthy();
  const diagnostics = screen.getByRole("complementary", { name: "Diagnostics" });
  expect(within(diagnostics).getByText("Simulated (demo mode)")).toBeTruthy();
});

test("shows_retry_attempt_and_cause", async () => {
  fakeShell(openRgbRefused);
  render(App);

  await waitForConnection("Retrying");
  expect(screen.getByText("Nothing answered at the OpenRGB server at 127.0.0.1:6742.")).toBeTruthy();
  expect(screen.getByText(/PrismForge tries again by itself.*Try 3 is next\./)).toBeTruthy();
  const panel = await openConnection();
  expect(within(panel).getByRole("button", { name: "Stop retrying" })).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/Attempt \d+ failed/);
});

test("explains_a_lost_connection_without_inventing_failed_attempts", async () => {
  fakeShell(openRgbLostWhileConnected);
  render(App);

  const notice = await screen.findByRole("status", { name: "Connection status" });
  expect(notice.textContent).toContain("The connection was lost: the server closed the connection.");
  expect(notice.textContent).toContain("Try 1 is next.");
  expect(screen.getByText("Retrying")).toBeTruthy();
  const list = screen.getByRole("navigation", { name: /Devices/ });
  expect(within(list).getAllByText("Unavailable")).toHaveLength(2);
  expect(screen.getByText(/Identify is off\. Not connected/)).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/Attempt \d+ failed|failed \d+ times/);
});

test("counts_retries_after_a_loss_as_the_core_does", async () => {
  fakeShell(openRgbLostRetryFailed);
  render(App);

  expect(await screen.findByText("Retrying")).toBeTruthy();
  const notice = screen.getByRole("status", { name: "Connection status" });
  expect(notice.textContent).toContain("did not answer the version request");
  expect(notice.textContent).toContain("Try 2 is next.");
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

  await shell.pushView({ revision: 10, session: demoConnected, applying: null, connect_apply: null, on_connect: "NotApplicable" });
  await shell.pushView({ revision: 9, session: openRgbRefused, applying: null, connect_apply: null, on_connect: "NotApplicable" });

  expect(await findDemoMarker()).toBeTruthy();
  expect(screen.queryByText(/^Retrying/)).toBeNull();
});

test("the_connection_screen_says_a_marked_profile_is_applied_on_connect", async () => {
  fakeShell(null);
  render(App);

  const promise = await screen.findByText(
    "PrismForge does not download, install or change OpenRGB. It writes nothing to your devices until you choose an action such as Identify, Apply effect or Apply profile, or until it connects while a profile is set to “Apply this profile when PrismForge connects”: that profile is applied as soon as the connection is made.",
  );
  expect(promise.closest("details")).toBeTruthy();
});

test("retrying_reads_as_waiting_not_as_a_failure", () => {
  expect(summarise(openRgbRefused).tone).toBe("waiting");
  expect(summarise(openRgbLostWhileConnected).tone).toBe("waiting");
  expect(summarise(openRgbOldServer).tone).toBe("problem");
});

function rememberedAt(host: string, port: number): LifecycleView {
  const view = structuredClone(lifecycleStartPath);
  if ("Ready" in view.settings) {
    view.settings.Ready.settings.address = { host, port };
  }
  return view;
}

test("the_connection_form_starts_from_the_last_openrgb_address", async () => {
  const shell = fakeShell(null, { lifecycle: () => rememberedAt("192.168.1.20", 6800), connect_openrgb: () => shell.view(openRgbConnecting) });
  render(App);

  const host = (await screen.findByRole("textbox", { name: "Host" })) as HTMLInputElement;
  await waitFor(() => expect(host.value).toBe("192.168.1.20"));
  expect((screen.getByRole("spinbutton", { name: "Port" }) as HTMLInputElement).value).toBe("6800");
  expect(screen.getByText("The address of your last Connect.")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Connect" }));
  expect(shell.calls.find((call) => call.command === "connect_openrgb")?.args).toEqual({ host: "192.168.1.20", port: 6800 });
});

test("waiting_for_openrgb_offers_a_way_back_and_demo_mode", async () => {
  const shell = fakeShell(openRgbRefused, { disconnect: () => shell.view(null), start_demo: () => shell.view(demoConnected) });
  render(App);
  const page = await screen.findByRole("main");
  await within(page).findByRole("heading", { name: "Waiting for OpenRGB" });

  expect(within(page).getByRole("button", { name: "Start demo mode" })).toBeTruthy();
  await fireEvent.click(within(page).getByRole("button", { name: "Stop and change the address" }));

  expect(shell.commands()).toContain("disconnect");
  expect(await screen.findByRole("textbox", { name: "Host" })).toBeTruthy();
});

test("an_invalid_port_is_announced_as_a_problem", async () => {
  fakeShell(null);
  render(App);
  const port = await screen.findByRole("spinbutton", { name: "Port" });

  await fireEvent.input(port, { target: { value: "0" } });

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("Enter a port from 1 to 65535.");
  expect(port.getAttribute("aria-invalid")).toBe("true");
  expect((screen.getByRole("button", { name: "Connect" }) as HTMLButtonElement).disabled).toBe(true);
});

test("the_prerequisites_are_one_line_with_the_rest_behind_details", async () => {
  fakeShell(null);
  render(App);

  const line = await screen.findByText("You need OpenRGB 1.0 or later with its SDK server on.");
  const details = line.parentElement?.querySelector("details");
  expect(details?.open).toBe(false);
  expect(within(details as HTMLElement).getByText(/Its SDK server is on/)).toBeTruthy();
});
