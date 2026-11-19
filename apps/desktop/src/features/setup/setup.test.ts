import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { Setup } from "../../lib/api/bindings/Setup";
import { fakeShell, refuse } from "../../test/backend";
import { errorSetupExceedsOutput } from "../../test/fixtures/errors";
import { demoConnected } from "../../test/fixtures/sessions";
import { setupDemoEmpty } from "../../test/fixtures/setups";
import { componentRow, inspector, openSetup, setupHandlers } from "../../test/setup";

function savedSetup(shell: ReturnType<typeof fakeShell>): Setup {
  const call = shell.calls.filter((each) => each.command === "save_setup").at(-1);
  return call!.args.setup as Setup;
}

function placementIn(setup: Setup, view: string, component: string) {
  return setup.views.find((each) => each.id === view)!.placements.find((placement) => placement.component === component)!;
}

function drawn(): string[] {
  return [...document.querySelectorAll<SVGElement>("[data-component]")].map((shape) => shape.dataset.component!);
}

test("the_pc_and_desk_tabs_each_show_their_own_components_in_the_drawing_and_the_list", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  const pc = within(list).getAllByRole("option").map((row) => row.getAttribute("aria-label") ?? row.querySelector(".pick-name")?.textContent);
  expect(screen.getByRole("tab", { name: "PC" }).getAttribute("aria-selected")).toBe("true");
  expect(pc).toEqual([
    "Front fan 1",
    "Front fan 2",
    "Front fan 3",
    "Rear fan",
    "Cooler left strip",
    "Cooler right strip",
    "Memory module 1",
    "Memory module 2",
    "Graphics card",
    "Board edge",
  ]);
  expect(drawn()).toEqual(["c-front-fan-1", "c-front-fan-2", "c-front-fan-3", "c-rear-fan", "c-cooler-left", "c-cooler-right", "c-memory-1", "c-memory-2", "c-graphics", "c-board-edge"]);

  await fireEvent.click(screen.getByRole("tab", { name: "Desk" }));

  const desk = await screen.findByRole("listbox", { name: "Components in the Desk view" });
  expect(within(desk).getAllByRole("option").map((row) => row.querySelector(".pick-name")?.textContent)).toEqual(["Desk light bar", "Graphics card"]);
  expect(drawn()).toEqual(["c-desk-bar", "c-graphics"]);
});

test("the_list_twin_says_what_each_component_is_where_it_is_wired_its_first_led_and_its_place", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  const fan = componentRow(list, "Front fan 1");
  const cooler = componentRow(list, "Cooler right strip");
  const memory = componentRow(list, "Memory module 1");

  expect(fan.textContent).toContain("Ring, 8 LEDs");
  expect(fan.textContent).toContain("Demo Motherboard, Addressable Header 1");
  expect(fan.textContent).toContain("Mirror: one control with Front fan 2, Front fan 3 and Rear fan. All four always show the same colours.");
  expect(fan.textContent).toContain("LED 1 at the top, runs clockwise");
  expect(fan.textContent).toContain("x 160, y 130, size 100, turned 0°");
  expect(cooler.textContent).toContain("Part 2 of 2 of a daisy chain on Addressable Header 2: LEDs 9 to 16");
  expect(memory.textContent).toContain("LED 1 at the top end, runs down");
  expect(memory.textContent).toContain("turned 90°");
});

test("leds_without_a_place_are_listed_as_not_placed_and_a_mirror_counts_once", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  await openSetup();

  const outputs = screen.getByRole("list", { name: "Outputs in the setup" });
  const hub = within(outputs).getByText(/^Addressable Header 1/).closest("li")!;

  expect(hub.textContent).toContain("4 fans mirrored: one control shown in 4 places");
  expect(hub.textContent).toContain("8 of 30 LEDs have a place. 22 are not placed and stay dark while an effect runs across the setup.");
});

test("focus_moves_with_the_arrow_keys_and_space_selects_apart_from_focus", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  const first = componentRow(list, "Front fan 1");
  first.focus();

  await fireEvent.keyDown(first, { key: "ArrowDown" });
  const second = componentRow(list, "Front fan 2");
  expect(document.activeElement).toBe(second);
  expect(second.getAttribute("aria-selected")).toBe("false");

  await fireEvent.keyDown(second, { key: " " });
  await fireEvent.keyDown(second, { key: "ArrowDown", shiftKey: true });

  expect(within(list).getAllByRole("option").filter((row) => row.getAttribute("aria-selected") === "true").map((row) => row.querySelector(".pick-name")?.textContent)).toEqual([
    "Front fan 2",
    "Front fan 3",
  ]);
  expect(document.getElementById("twin-count")?.textContent).toBe("2 components selected");

  await fireEvent.keyDown(componentRow(list, "Front fan 3"), { key: "Enter" });
  expect(within(list).getAllByRole("option").filter((row) => row.getAttribute("aria-selected") === "true")).toHaveLength(1);

  await fireEvent.keyDown(componentRow(list, "Front fan 3"), { key: "a", ctrlKey: true });
  expect(within(list).getAllByRole("option").every((row) => row.getAttribute("aria-selected") === "true")).toBe(true);
});

test("keyboard_moves_turns_and_flips_the_selection_and_undo_restores_it", async () => {
  const shell = fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  const graphics = componentRow(list, "Graphics card");
  await fireEvent.click(graphics);
  graphics.focus();

  await fireEvent.keyDown(graphics, { key: "ArrowRight", altKey: true });
  await fireEvent.keyDown(graphics, { key: "ArrowUp", altKey: true, shiftKey: true });
  await fireEvent.keyDown(graphics, { key: "]" });
  await fireEvent.keyDown(graphics, { key: "H", shiftKey: true });
  const edited = within(inspector()).getByRole("spinbutton", { name: "X" }) as HTMLInputElement;

  expect(edited.value).toBe("570");
  expect((within(inspector()).getByRole("spinbutton", { name: "Y" }) as HTMLInputElement).value).toBe("420");
  expect((within(inspector()).getByRole("spinbutton", { name: "Turn" }) as HTMLInputElement).value).toBe("15");
  expect(within(inspector()).getByRole("button", { name: "Flip" }).getAttribute("aria-pressed")).toBe("true");

  await fireEvent.keyDown(graphics, { key: "z", ctrlKey: true });
  await fireEvent.keyDown(graphics, { key: "z", ctrlKey: true });
  expect((within(inspector()).getByRole("spinbutton", { name: "Turn" }) as HTMLInputElement).value).toBe("0");
  await fireEvent.keyDown(graphics, { key: "y", ctrlKey: true });
  expect((within(inspector()).getByRole("spinbutton", { name: "Turn" }) as HTMLInputElement).value).toBe("15");

  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
  expect(placementIn(savedSetup(shell), "v-pc", "c-graphics")).toMatchObject({ x: 570, y: 420, rotation: 15, flipped: false });
});

test("several_selected_components_move_together_with_the_buttons", async () => {
  const shell = fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Front fan 1"));
  await fireEvent.click(componentRow(list, "Rear fan"), { ctrlKey: true });

  const panel = inspector();
  expect(within(panel).getByRole("heading", { name: "2 components selected" })).toBeTruthy();
  await fireEvent.click(within(panel).getByRole("button", { name: "Move down" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));

  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
  expect(placementIn(savedSetup(shell), "v-pc", "c-front-fan-1")).toMatchObject({ x: 160, y: 140 });
  expect(placementIn(savedSetup(shell), "v-pc", "c-rear-fan")).toMatchObject({ x: 860, y: 140 });
});

test("number_fields_set_the_place_and_each_change_is_one_undo_step", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Rear fan"));
  const x = within(inspector()).getByRole("spinbutton", { name: "X" }) as HTMLInputElement;

  await fireEvent.change(x, { target: { value: "640" } });
  await fireEvent.change(within(inspector()).getByRole("spinbutton", { name: "Turn" }), { target: { value: "45" } });

  const undo = screen.getByRole("button", { name: "Undo" });
  expect(undo.getAttribute("aria-description") ?? document.getElementById(undo.getAttribute("aria-describedby") ?? "")?.textContent).toMatch(/Turn Rear fan/);
  await fireEvent.click(undo);
  expect(x.value).toBe("640");
  expect((within(inspector()).getByRole("spinbutton", { name: "Turn" }) as HTMLInputElement).value).toBe("0");
});

test("undo_and_save_never_claim_a_change_to_the_lights", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  await openSetup();

  const bar = screen.getByRole("region", { name: "Setup actions" });

  expect(bar.textContent).toContain("Undo and redo change this drawing only. Your lights stay as they are.");
  expect(bar.textContent).toContain("Saved");
  expect(bar.textContent).not.toMatch(/undo.*(light|device|hardware)s? (back|change)/i);
});

test("save_shows_unsaved_changes_and_a_refusal_with_the_reason_beside_the_field", async () => {
  fakeShell(demoConnected, setupHandlers(undefined, { save_setup: refuse(errorSetupExceedsOutput) }));
  render(App);
  const list = await openSetup();
  await fireEvent.click(componentRow(list, "Cooler left strip"));

  await fireEvent.change(within(inspector()).getByRole("spinbutton", { name: "LED count" }), { target: { value: "9" } });

  const bar = screen.getByRole("region", { name: "Setup actions" });
  expect(bar.textContent).toContain("Unsaved changes");
  expect(within(inspector()).getByText("The parts on Addressable Header 2 need 17 LEDs, and it has 16. Use fewer LEDs.")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("The setup was not saved.");
  expect(alert.textContent).toContain("The parts on Addressable Header 2 need 17 LEDs, and it has 16.");
  expect(bar.textContent).toContain("Unsaved changes");
});

test("a_detected_zone_is_placed_from_the_list_as_a_wired_component", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  await openSetup();
  const detected = screen.getByRole("region", { name: "Detected devices" });
  const header = within(detected).getByRole("group", { name: "Addressable Header 1 on Demo Motherboard" });

  await fireEvent.click(within(header).getByRole("button", { name: "Place in the PC view…" }));
  const form = within(header).getByRole("group", { name: "Place Addressable Header 1" });
  await fireEvent.change(within(form).getByRole("combobox", { name: "Shape" }), { target: { value: "Ring" } });
  await fireEvent.change(within(form).getByRole("spinbutton", { name: "LED count" }), { target: { value: "12" } });
  await fireEvent.click(within(form).getByRole("button", { name: "Place" }));

  const list = screen.getByRole("listbox", { name: "Components in the PC view" });
  expect(within(list).getByRole("option", { name: "Addressable Header 1" }).textContent).toContain("Ring, 12 LEDs");
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
  const saved = savedSetup(shell);
  expect(saved.outputs).toEqual([
    {
      id: "o-1",
      device: "demo-motherboard",
      zone: 1,
      label: "Demo Motherboard",
      zone_name: "Addressable Header 1",
      first_led: 0,
      branches: [["c-1"]],
    },
  ]);
  expect(saved.components[0]).toEqual({ id: "c-1", name: "Addressable Header 1", geometry: { Ring: { leds: 12, first_angle: 0, turn: "Clockwise" } } });
});

test("a_second_component_on_a_zone_asks_whether_it_mirrors_or_continues_the_chain", async () => {
  const shell = fakeShell(demoConnected, setupHandlers());
  render(App);
  await openSetup();
  const detected = screen.getByRole("region", { name: "Detected devices" });
  const header = within(detected).getByRole("group", { name: "Addressable Header 2 on Demo Motherboard" });

  await fireEvent.click(within(header).getByRole("button", { name: "Place in the PC view…" }));
  const form = within(header).getByRole("group", { name: "Place Addressable Header 2" });
  expect(within(form).getByRole("radio", { name: /^Next part of the daisy chain/ })).toBeTruthy();
  await fireEvent.click(within(form).getByRole("radio", { name: /^Mirror/ }));
  await fireEvent.click(within(form).getByRole("button", { name: "Place" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));

  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
  expect(savedSetup(shell).outputs.find((output) => output.id === "o-cooler")!.branches).toEqual([["c-cooler-left", "c-cooler-right"], ["c-1"]]);
});
