import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { Setup } from "../../lib/api/bindings/Setup";
import { fakeShell } from "../../test/backend";
import { acrossDemoTower } from "../../test/fixtures/acrossReports";
import { previewDemoTower } from "../../test/fixtures/acrossPreviews";
import { savedNew } from "../../test/fixtures/saves";
import { demoAcrossRunning, demoConnected } from "../../test/fixtures/sessions";
import { setupDemoEmpty } from "../../test/fixtures/setups";
import { openSetup, setupHandlers } from "../../test/setup";

test("a_detected_zone_becomes_a_placed_fan_runs_an_effect_across_the_setup_and_is_saved_in_a_profile", async () => {
  const shell = fakeShell(
    demoConnected,
    setupHandlers(setupDemoEmpty, {
      preview_across: () => previewDemoTower,
      apply_across: () => {
        void shell.push(demoAcrossRunning);
        return acrossDemoTower;
      },
      create_profile: () => savedNew,
    }),
  );
  render(App);
  await openSetup();

  const header = within(screen.getByRole("region", { name: "Detected devices" })).getByRole("group", { name: "Addressable Header 1 on Demo Motherboard" });
  await fireEvent.click(within(header).getByRole("button", { name: "Place in the PC view…" }));
  const form = within(header).getByRole("group", { name: "Place Addressable Header 1" });
  await fireEvent.change(within(form).getByRole("combobox", { name: "Shape" }), { target: { value: "Ring" } });
  await fireEvent.change(within(form).getByRole("spinbutton", { name: "LED count" }), { target: { value: "12" } });
  await fireEvent.click(within(form).getByRole("button", { name: "Place" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  await waitFor(() => expect(screen.getByRole("region", { name: "Setup actions" }).textContent).toContain("Saved just now"));

  const across = screen.getByRole("region", { name: "Effect across the setup" });
  await fireEvent.change(within(across).getByRole("combobox", { name: "Effect" }), { target: { value: "rainbow-wave" } });
  await fireEvent.click(within(across).getByRole("button", { name: "Apply across the PC view" }));
  await within(across).findByRole("status", { name: "Results for each target" });

  await fireEvent.click(within(screen.getByRole("navigation", { name: "Main" })).getByRole("button", { name: "Profiles" }));
  const list = await screen.findByRole("navigation", { name: "Profiles" });
  await fireEvent.click(within(list).getByRole("button", { name: "Save current lighting…" }));
  await screen.findByRole("dialog", { name: "Save the current lighting" });
  await fireEvent.input(screen.getByRole("textbox", { name: "Profile name" }), { target: { value: "Across" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save as new profile" }));

  const saved = shell.calls.find((call) => call.command === "save_setup")!.args.setup as Setup;
  expect(saved.outputs).toEqual([
    { id: "o-1", device: "demo-motherboard", zone: 1, label: "Demo Motherboard", zone_name: "Addressable Header 1", first_led: 0, branches: [["c-1"]] },
  ]);
  const applied = shell.calls.find((call) => call.command === "apply_across")!.args;
  expect({ view: applied.view, flow: applied.flow, effect: (applied.effect as { effect: string }).effect }).toEqual({
    view: "v-pc",
    flow: { Along: { angle: 0 } },
    effect: "rainbow-wave",
  });
  await waitFor(() => expect(shell.commands()).toContain("create_profile"));
  expect(shell.commands().filter((command) => ["save_setup", "apply_across", "create_profile"].includes(command))).toEqual(["save_setup", "apply_across", "create_profile"]);
});
