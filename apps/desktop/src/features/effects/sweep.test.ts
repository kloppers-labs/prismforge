import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { button, chooseTarget, effectHandlers } from "../../test/effects";
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
  const bar = screen.getByRole("region", { name: "Effect actions" });

  await waitFor(() => expect(within(bar).getByText("Runs while PrismForge runs and is connected.")).toBeTruthy());
  const how = within(bar).getByText("How effects run").closest("details") as HTMLDetailsElement;
  expect(how.open).toBe(false);
  expect(within(how).getByText(/no effect runs until you apply one, or until a profile marked to apply on connect is applied/)).toBeTruthy();
});
