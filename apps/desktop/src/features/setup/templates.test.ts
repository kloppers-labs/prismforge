import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { Setup } from "../../lib/api/bindings/Setup";
import { fakeShell } from "../../test/backend";
import { demoConnected } from "../../test/fixtures/sessions";
import { setupDemoEmpty, setupDemoTower } from "../../test/fixtures/setups";
import { openSetup, setupHandlers } from "../../test/setup";

function saved(shell: ReturnType<typeof fakeShell>): Setup {
  return shell.calls.filter((call) => call.command === "save_setup").at(-1)!.args.setup as Setup;
}

async function save(shell: ReturnType<typeof fakeShell>) {
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
}

function names(list: HTMLElement): string[] {
  return within(list)
    .getAllByRole("option")
    .map((row) => row.querySelector(".pick-name")?.textContent ?? "");
}

async function map(component: string, output: string, joining?: RegExp) {
  const unwired = screen.getByRole("region", { name: "Not wired yet" });
  const row = within(unwired).getByRole("group", { name: component });
  await fireEvent.change(within(row).getByRole("combobox", { name: "Output" }), { target: { value: output } });
  if (joining) {
    await fireEvent.click(within(row).getByRole("radio", { name: joining }));
  }
  await fireEvent.click(within(row).getByRole("button", { name: "Wire" }));
}

test("the_tower_template_proposes_components_and_wires_none_of_them", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  const list = await openSetup();

  await fireEvent.click(screen.getByRole("button", { name: "Add the Tower PC template" }));

  expect(names(list)).toEqual([
    "Front fan 1",
    "Front fan 2",
    "Front fan 3",
    "Rear fan",
    "Cooler strip 1",
    "Cooler strip 2",
    "Memory module 1",
    "Memory module 2",
    "Graphics card",
    "Board edge",
    "Case",
  ]);
  expect(within(list).getAllByRole("option").slice(0, 10).every((row) => row.textContent?.includes("Not wired to an output"))).toBe(true);
  expect(within(list).getByRole("option", { name: "Case" }).textContent).toContain("No LEDs");
  expect(screen.getByRole("button", { name: "Undo" })).toBeTruthy();
  await save(shell);
  expect(saved(shell).outputs).toEqual([]);
  expect(saved(shell).components).toHaveLength(10);
});

test("each_template_component_is_wired_only_by_the_users_choice_a_hub_as_mirrors_and_a_cooler_as_a_chain", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  await openSetup();
  await fireEvent.click(screen.getByRole("button", { name: "Add the Tower PC template" }));

  await map("Front fan 1", "demo-motherboard|1");
  await map("Front fan 2", "demo-motherboard|1", /^Mirror/);
  await map("Rear fan", "demo-motherboard|1", /^Mirror/);
  await map("Cooler strip 1", "demo-motherboard|2");
  await map("Cooler strip 2", "demo-motherboard|2", /^Next part of the daisy chain/);
  await save(shell);

  const outputs = saved(shell).outputs.map((output) => [output.device, output.zone, output.branches]);
  const ids = Object.fromEntries(saved(shell).components.map((component) => [component.name, component.id]));
  expect(outputs).toEqual([
    ["demo-motherboard", 1, [[ids["Front fan 1"]], [ids["Front fan 2"]], [ids["Rear fan"]]]],
    ["demo-motherboard", 2, [[ids["Cooler strip 1"], ids["Cooler strip 2"]]]],
  ]);
  const unwired = screen.getByRole("region", { name: "Not wired yet" });
  expect(within(unwired).getAllByRole("group").map((row) => row.getAttribute("aria-label"))).toEqual([
    "Front fan 3",
    "Memory module 1",
    "Memory module 2",
    "Graphics card",
    "Board edge",
  ]);
});

test("the_wire_button_waits_until_the_user_says_mirror_or_chain", async () => {
  fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  await openSetup();
  await fireEvent.click(screen.getByRole("button", { name: "Add the Tower PC template" }));
  await map("Front fan 1", "demo-motherboard|1");

  const row = within(screen.getByRole("region", { name: "Not wired yet" })).getByRole("group", { name: "Front fan 2" });
  await fireEvent.change(within(row).getByRole("combobox", { name: "Output" }), { target: { value: "demo-motherboard|1" } });

  expect((within(row).getByRole("button", { name: "Wire" }) as HTMLButtonElement).disabled).toBe(true);
  expect(row.textContent).toContain("PrismForge cannot see the wiring.");
});

test("the_desk_template_fills_the_desk_view", async () => {
  fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  await openSetup();
  await fireEvent.click(screen.getByRole("tab", { name: "Desk" }));

  await fireEvent.click(screen.getByRole("button", { name: "Add the Desk template" }));

  const desk = screen.getByRole("listbox", { name: "Components in the Desk view" });
  expect(names(desk)).toEqual(["Monitor edge", "Desk edge strip", "Keyboard", "Light bar", "Mouse logo", "Desk", "Monitor", "PC"]);
});

test("a_component_of_the_other_view_is_placed_here_from_the_list", async () => {
  fakeShell(demoConnected, setupHandlers(setupDemoTower));
  render(App);
  await openSetup();
  await fireEvent.click(screen.getByRole("tab", { name: "Desk" }));

  const elsewhere = screen.getByRole("region", { name: "Not in this view" });
  await fireEvent.click(within(within(elsewhere).getByRole("group", { name: "Front fan 1" })).getByRole("button", { name: "Place in the Desk view" }));

  const desk = screen.getByRole("listbox", { name: "Components in the Desk view" });
  expect(names(desk)).toContain("Front fan 1");
  expect(within(desk).getByRole("option", { name: "Front fan 1" }).getAttribute("aria-selected")).toBe("true");
});
