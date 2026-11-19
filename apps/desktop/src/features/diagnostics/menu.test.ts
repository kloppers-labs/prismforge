import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { beforeEach, expect, test } from "vitest";
import App from "../../App.svelte";
import { fakeShell } from "../../test/backend";
import { demoEffectsRunning } from "../../test/fixtures/sessions";
import { itemLabels, openMenu, pick } from "../../test/menus";

const SECTIONS = ["Profile store", "Backend", "Last error", "Counters", "Output", "Device references", "Recent events, newest first"];
let copied: string[] = [];

beforeEach(() => {
  copied = [];
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: (text: string) => (copied.push(text), Promise.resolve()) } });
});

async function openDiagnostics() {
  fakeShell(demoEffectsRunning);
  render(App);
  await screen.findByRole("navigation", { name: /Devices/ });
  await fireEvent.click(await screen.findByRole("button", { name: "Diagnostics" }));
  const drawer = screen.getByRole("complementary", { name: "Diagnostics" });
  await within(drawer).findByText(/store-folder\/demo-store/);
  return drawer;
}

function blocks(text: string): string[] {
  return text.split("\n\n").slice(1);
}

test("each_section_copies_only_its_part_of_copy_as_text", async () => {
  const drawer = await openDiagnostics();
  await fireEvent.click(within(drawer).getByRole("button", { name: "Copy as text" }));
  const whole = blocks(copied[0] ?? "");

  for (const name of SECTIONS) {
    await fireEvent.click(within(drawer).getByRole("button", { name: `Copy the ${name} section` }));
    const part = copied.at(-1) ?? "";
    expect(part.startsWith("PrismForge diagnostics\n\n")).toBe(true);
    expect(blocks(part)).toHaveLength(1);
    expect(whole).toContain(blocks(part)[0]);
  }
  expect(copied.slice(1).join("\n")).not.toMatch(/store-folder/);
  expect(await within(drawer).findByText("Copied. Paste it into your report.")).toBeTruthy();
});

test("the_section_menu_copies_like_the_section_button_and_copy_as_text", async () => {
  const drawer = await openDiagnostics();
  const backend = within(drawer).getByRole("region", { name: "Backend" });
  await fireEvent.click(within(backend).getByRole("button", { name: "Copy the Backend section" }));
  await fireEvent.click(within(drawer).getByRole("button", { name: "Copy as text" }));

  expect(itemLabels(await openMenu(backend, "Actions for Backend"))).toEqual(["Copy section", "Copy as text"]);
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  await pick(backend, "Actions for Backend", "Copy section");
  await pick(backend, "Actions for Backend", "Copy as text");

  expect(copied).toHaveLength(4);
  expect(copied[2]).toBe(copied[0]);
  expect(copied[3]).toBe(copied[1]);
});

test("a_section_menu_opens_from_the_keyboard_on_its_copy_button", async () => {
  const drawer = await openDiagnostics();
  const button = within(drawer).getByRole("button", { name: "Copy the Counters section" });
  button.focus();

  await fireEvent.keyDown(button, { key: "ContextMenu" });

  expect(screen.getByRole("menu", { name: "Actions for Counters" })).toBeTruthy();
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(document.activeElement).toBe(button);
  expect(screen.getByRole("complementary", { name: "Diagnostics" })).toBeTruthy();
});
