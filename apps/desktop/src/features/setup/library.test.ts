import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import App from "../../App.svelte";
import type { Setup } from "../../lib/api/bindings/Setup";
import { fakeShell } from "../../test/backend";
import { demoConnected } from "../../test/fixtures/sessions";
import { setupDemoEmpty, setupDemoTower } from "../../test/fixtures/setups";
import { itemLabels, openMenu, pick } from "../../test/menus";
import { componentRow, inspector, openSetup, setupHandlers } from "../../test/setup";

function saved(shell: ReturnType<typeof fakeShell>): Setup {
  return shell.calls.filter((call) => call.command === "save_setup").at(-1)!.args.setup as Setup;
}

async function save(shell: ReturnType<typeof fakeShell>) {
  await fireEvent.click(screen.getByRole("button", { name: "Save setup" }));
  await waitFor(() => expect(shell.commands()).toContain("save_setup"));
}

async function place(group: string, form: string, shape: string, leds?: string) {
  const detected = screen.getByRole("region", { name: "Detected devices" });
  const entry = within(detected).getByRole("group", { name: group });
  await fireEvent.click(within(entry).getByRole("button", { name: /^Place in the/ }));
  const place = within(entry).getByRole("group", { name: form });
  await fireEvent.change(within(place).getByRole("combobox", { name: "Shape" }), { target: { value: shape } });
  if (leds) {
    await fireEvent.change(within(place).getByRole("spinbutton", { name: "LED count" }), { target: { value: leds } });
  }
  await fireEvent.click(within(place).getByRole("button", { name: "Place" }));
}

async function addOutline(kind: string) {
  const panel = screen.getByRole("region", { name: "Outlines without LEDs" });
  await fireEvent.change(within(panel).getByRole("combobox", { name: "Outline" }), { target: { value: kind } });
  await fireEvent.click(within(panel).getByRole("button", { name: "Add the outline" }));
}

test("a_block_is_a_lit_shape_with_a_frame_round_its_leds", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  const list = await openSetup();

  await place("Demo Graphics Card", "Place Demo Graphics Card", "BlockStrip");

  const row = componentRow(list, "Demo Graphics Card");
  expect(row.textContent).toContain("Strip, 12 LEDs, in a frame");
  expect(document.querySelector('[data-component="c-1"] .frame')).not.toBeNull();
  await fireEvent.change(within(inspector()).getByRole("spinbutton", { name: "Frame width" }), { target: { value: "140" } });
  await save(shell);
  expect(saved(shell).components[0]).toMatchObject({ geometry: { Strip: { leds: 12 } }, frame: { width: 140, height: 40 } });
  expect(saved(shell).outputs[0]!.branches).toEqual([["c-1"]]);
});

test("a_free_line_is_placed_and_its_points_are_edited_without_a_drag", async () => {
  const shell = fakeShell(demoConnected, setupHandlers(setupDemoEmpty));
  render(App);
  await openSetup();
  await place("Demo Light Bar", "Place Demo Light Bar", "Line", "10");
  const points = within(inspector()).getByRole("group", { name: "Points of the line" });

  await fireEvent.change(within(points).getByRole("spinbutton", { name: "Point 2, x" }), { target: { value: "80" } });
  await fireEvent.change(within(points).getByRole("spinbutton", { name: "Point 3, y" }), { target: { value: "90" } });
  await fireEvent.click(within(points).getByRole("button", { name: "Add a point" }));
  await fireEvent.click(within(points).getByRole("button", { name: "Remove point 1" }));
  await fireEvent.click(screen.getByRole("button", { name: "Undo" }));
  await save(shell);

  expect(saved(shell).components[0]!.geometry).toEqual({
    Line: {
      leds: 10,
      points: [
        { x: 0, y: 0 },
        { x: 80, y: 0 },
        { x: 100, y: 90 },
        { x: 120, y: 180 },
      ],
      first: "AtStart",
    },
  });
});

test("an_unlit_outline_is_listed_moved_resized_and_saved_without_any_output", async () => {
  const shell = fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();

  await addOutline("Case");
  const row = within(list).getByRole("option", { name: "Case" });
  expect(row.textContent).toContain("Outline of a case. No LEDs: it is drawn only and nothing is wired to it.");
  expect(row.getAttribute("aria-selected")).toBe("true");
  await fireEvent.keyDown(row, { key: "ArrowRight", altKey: true });
  await fireEvent.change(within(inspector()).getByRole("spinbutton", { name: "Width" }), { target: { value: "300" } });
  await save(shell);

  const outline = saved(shell).views.find((view) => view.id === "v-pc")!.outlines;
  expect(outline).toEqual([{ id: "u-1", name: "Case", kind: "Case", x: 510, y: 350, width: 300, height: 560, rotation: 0 }]);
  expect(saved(shell).outputs).toEqual(setupDemoTower.setup.outputs);
  expect(document.querySelector('[data-outline="u-1"] .led')).toBeNull();
});

test("an_outline_row_menu_offers_its_editor_actions_and_delete_asks_first", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  await addOutline("Monitor");
  const row = within(list).getByRole("option", { name: "Monitor" });

  const menu = await openMenu(row, "Actions for Monitor");
  expect(itemLabels(menu)).toEqual(["Select only this outline", "Turn right 15°", "Delete outline…"]);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  await pick(row, "Actions for Monitor", "Delete outline…");

  const dialog = await screen.findByRole("dialog", { name: "Delete Monitor?" });
  await waitFor(() => expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Keep outline" })));
});

test("adding_an_outline_is_one_undo_step", async () => {
  fakeShell(demoConnected, setupHandlers());
  render(App);
  const list = await openSetup();
  await addOutline("Desk");

  await fireEvent.click(screen.getByRole("button", { name: "Undo" }));

  expect(within(list).queryByRole("option", { name: "Desk" })).toBeNull();
});
