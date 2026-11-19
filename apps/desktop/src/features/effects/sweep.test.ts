import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers } from "../../test/effects";
import { go } from "../../test/shell";
import { demoConnected, demoEffectsRunning } from "../../test/fixtures/sessions";

function tabStops(): HTMLElement[] {
  const all = [...document.querySelectorAll<HTMLElement>("button, input, summary, [tabindex]")];
  const radios = new Set<string>();
  return all.filter((element) => {
    if (element.tabIndex < 0 || (element as HTMLButtonElement).disabled || element.closest("[inert], details:not([open]) > :not(summary)")) {
      return false;
    }
    if (element instanceof HTMLInputElement && element.type === "radio") {
      const group = element.name;
      const checked = document.querySelector<HTMLInputElement>(`input[type=radio][name="${group}"]:checked`);
      if (radios.has(group) || (checked && checked !== element)) {
        return false;
      }
      radios.add(group);
    }
    return true;
  });
}

test("apply_sits_in_the_effect_bar_early_in_the_tab_order", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));

  const bar = screen.getByRole("region", { name: "Effect actions" });
  expect(within(bar).getByRole("button", { name: "Apply effect" })).toBeTruthy();
  expect(within(bar).getByRole("button", { name: "Stop effect" })).toBeTruthy();
  const position = tabStops().indexOf(button("Apply effect"));
  expect(position).toBeGreaterThan(-1);
  expect(position).toBeLessThan(14);
});

test("the_whole_effect_card_chooses_the_effect", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  await fireEvent.click(screen.getByText("The colour wheel travels along the LEDs."));

  expect((screen.getByRole("radio", { name: "Rainbow wave" }) as HTMLInputElement).checked).toBe(true);
});

test("the_bar_names_live_and_draft", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");
  const bar = screen.getByRole("region", { name: "Effect actions" });

  expect(within(bar).getByText("Live")).toBeTruthy();
  expect(bar.textContent).toMatch(/Changes go to Demo Graphics Card as you make them\./);

  await fireEvent.click(screen.getByRole("radio", { name: "Breathing" }));

  expect(within(bar).getByText("Draft")).toBeTruthy();
  expect(bar.textContent).toMatch(/Rainbow wave runs now\. Apply effect replaces it\./);
});

test("solid_colour_starts_with_a_colour_and_can_be_applied", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  await fireEvent.click(screen.getByRole("radio", { name: "Solid colour" }));

  expect((screen.getByRole("textbox", { name: "Colour" }) as HTMLInputElement).value).toBe("#ffffff");
  expect(button("Apply effect").disabled).toBe(false);
});

test("a_device_left_to_another_program_offers_hand_back_in_its_notice", async () => {
  const shell = fakeShell(demoEffectsRunning, effectHandlers({ clear_external: () => null }));
  render(App);
  await chooseTarget("Demo Memory Module", "Whole device", 1);

  const notice = screen.getByRole("status", { name: "Left to another program" });
  await fireEvent.click(within(notice).getByRole("button", { name: "Hand back to PrismForge" }));

  expect(shell.commands()).toContain("clear_external");
  expect(screen.getByRole("group", { name: "Effect" }).hasAttribute("disabled")).toBe(true);
});

test("the_target_list_says_controlled_by_only_when_something_controls", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  expect(document.body.textContent).not.toMatch(/Owner/);
});

test("the_effects_lifecycle_text_sits_behind_details_under_one_line", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");
  await waitFor(() => expect(screen.getByText("Runs while PrismForge runs and is connected.")).toBeTruthy());
  const how = screen.getByText("How effects run").closest("details") as HTMLDetailsElement;
  expect(how.open).toBe(false);
  expect(within(how).getByText(/no effect runs until you apply one, or until a profile marked to apply on connect is applied/)).toBeTruthy();
});

test("the_bar_holds_only_the_target_the_actions_the_mode_and_the_output", async () => {
  fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  await fireEvent.click(button("Apply effect"));
  const bar = screen.getByRole("region", { name: "Effect actions" });

  await waitFor(() => expect(screen.getByRole("status", { name: "Effect result" }).textContent).toMatch(/accepted Rainbow wave/));
  expect(bar.contains(screen.getByRole("status", { name: "Effect result" }))).toBe(false);
  expect(within(bar).queryByText("How effects run")).toBeNull();
  expect(screen.getAllByText(/Runs while PrismForge runs and is connected\./)).toHaveLength(1);
});

test("the_lifecycle_text_says_a_restore_follows_a_stop_or_a_quit_only", async () => {
  fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  const how = screen.getByText("How effects run").closest("details") as HTMLElement;
  expect(how.textContent).toMatch(/When you stop an effect or quit, PrismForge asks the demo backend to put back the earlier lighting\./);
  expect(how.textContent).not.toMatch(/When an effect stops/);
});

test("a_whole_device_target_names_the_zone_effect_that_runs_and_what_stop_does", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  await chooseTarget("Demo Motherboard");
  const bar = screen.getByRole("region", { name: "Effect actions" });

  expect(bar.textContent).toMatch(/Moving highlight runs on Addressable Header 1\. Stop effect stops it\./);
  await fireEvent.click(screen.getByRole("radio", { name: "Gradient" }));
  expect(bar.textContent).toMatch(/Apply effect replaces it with one effect on the whole device\./);
});

test("the_target_list_keeps_one_tab_stop_when_the_chosen_target_is_gone", async () => {
  const shell = fakeShell(demoConnected, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  const gone = structuredClone(demoConnected);
  gone.devices = gone.devices.filter((device) => device.descriptor.name !== "Demo Graphics Card");
  await shell.push(gone);

  await screen.findByText(/not in the device list any more/);
  const targets = screen.getByRole("navigation", { name: "Targets" });
  expect(within(targets).getAllByRole("option").filter((row) => row.tabIndex === 0)).toHaveLength(1);
});

test("while_an_effect_runs_the_page_says_once_that_it_runs_while_prismforge_runs", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  expect(screen.getAllByText(/runs (only )?while prismforge runs and is connected/i)).toHaveLength(1);
});

test("a_live_edit_never_scrolls_the_page_but_an_apply_press_shows_its_result", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const scrolled = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});
  fakeShell(demoEffectsRunning, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  scrolled.mockClear();

  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "40" } });
  await new Promise((resolve) => setTimeout(resolve, 600));
  await fireEvent.input(screen.getByRole("spinbutton", { name: "Brightness" }), { target: { value: "45" } });
  await new Promise((resolve) => setTimeout(resolve, 600));
  expect(scrolled).not.toHaveBeenCalled();

  await fireEvent.click(button("Apply effect"));
  await waitFor(() => expect(scrolled).toHaveBeenCalled());
  scrolled.mockRestore();
});

test("switching_targets_or_returning_to_effects_never_scrolls_only_an_apply_press_does", async () => {
  Element.prototype.scrollIntoView ??= () => {};
  const scrolled = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});
  fakeShell(demoConnected, effectHandlers({ apply_effect: () => null }));
  render(App);
  await chooseTarget("Demo Graphics Card");
  await fireEvent.click(screen.getByRole("radio", { name: "Rainbow wave" }));
  await fireEvent.click(button("Apply effect"));
  await waitFor(() => expect(scrolled).toHaveBeenCalledTimes(1));
  scrolled.mockClear();

  await chooseTarget("Demo Light Bar");
  await chooseTarget("Demo Graphics Card");
  await go("Devices");
  await chooseTarget("Demo Graphics Card");
  await new Promise((resolve) => setTimeout(resolve, 50));
  expect(scrolled).not.toHaveBeenCalled();

  await fireEvent.click(screen.getByRole("radio", { name: "Breathing" }));
  await fireEvent.click(button("Apply effect"));
  await waitFor(() => expect(scrolled).toHaveBeenCalledTimes(1));
  scrolled.mockRestore();
});

test("the_owner_line_names_the_demo_backend_in_demo_mode", async () => {
  fakeShell(demoEffectsRunning, effectHandlers());
  render(App);
  await chooseTarget("Demo Graphics Card");

  expect(screen.getByText("A PrismForge effect, computed by PrismForge and sent to the demo backend. Nothing reaches hardware.")).toBeTruthy();
  expect(document.body.textContent).not.toMatch(/sent to the device/);
});
