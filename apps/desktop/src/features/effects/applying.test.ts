import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers } from "../../test/effects";
import { demoConnected } from "../../test/fixtures/sessions";
import { viewApplying, viewConnectApplied } from "../../test/fixtures/views";

const REASON = /PrismForge is applying the profile Evening\. Wait until it finishes\./;

test("output_controls_are_off_with_a_reason_while_a_profile_applies", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  expect(button("Apply effect").disabled).toBe(false);

  await shell.pushView({ ...viewApplying, revision: 50 });

  await waitFor(() => expect(button("Apply effect").disabled).toBe(true));
  expect(screen.getAllByText(REASON).length).toBeGreaterThan(0);
  expect(button("Stop effect").disabled).toBe(true);
  expect(screen.getAllByRole("button", { name: /^Use / }).every((use) => (use as HTMLButtonElement).disabled)).toBe(true);
  expect(button("Another program controls this device").disabled).toBe(true);
  expect(button("Set frame rate").disabled).toBe(true);

  await shell.pushView({ ...viewApplying, applying: null, revision: 51 });
  await waitFor(() => expect(button("Apply effect").disabled).toBe(false));
});

test("identify_is_off_while_a_profile_applies", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  const list = await screen.findByRole("navigation", { name: /Devices/ });
  await fireEvent.click(within(list).getByRole("button", { name: /^Demo Graphics Card/ }));

  await shell.pushView({ ...viewApplying, revision: 50 });

  await waitFor(() => expect(button("Identify device").disabled).toBe(true));
  expect(screen.getByText(REASON)).toBeTruthy();
});

test("the_apply_on_connect_result_is_shown_per_target", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });

  await shell.pushView({ ...viewConnectApplied, revision: 50 });

  const notice = await screen.findByRole("status", { name: "Applied on connect" });
  expect(notice.textContent).toMatch(/PrismForge applied Evening when it connected\. All 4 settings were accepted\./);
  expect(within(notice).getAllByRole("listitem")).toHaveLength(4);
  await fireEvent.click(within(notice).getByRole("button", { name: "Dismiss" }));
  expect(screen.queryByRole("status", { name: "Applied on connect" })).toBeNull();
});

test("an_apply_on_connect_with_a_skipped_setting_is_not_called_applied", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  const partly = structuredClone(viewConnectApplied);
  const outcome = partly.connect_apply?.outcome;
  if (outcome && "Applied" in outcome) {
    outcome.Applied.report.frame_rate = { Refused: { error: { Session: "Busy" } } };
  }

  await shell.pushView({ ...partly, revision: 50 });

  const notice = await screen.findByRole("status", { name: "Applied on connect" });
  expect(notice.textContent).toMatch(/PrismForge applied Evening in part when it connected\. The frame rate was refused\./);
});
