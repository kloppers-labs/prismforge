import { fireEvent, screen, within } from "@testing-library/svelte";
import { expect } from "vitest";

export function menuItems(menu: HTMLElement): HTMLElement[] {
  return [...menu.querySelectorAll<HTMLElement>("[data-item]")];
}

export function itemLabels(menu: HTMLElement): string[] {
  return menuItems(menu).map((item) => item.querySelector("[id$='-label']")?.textContent ?? "");
}

export async function openMenu(origin: HTMLElement, name: string | RegExp) {
  await fireEvent.contextMenu(origin);
  return screen.getByRole("menu", { name });
}

export async function pick(origin: HTMLElement, menuName: string | RegExp, item: string | RegExp, role: "menuitem" | "menuitemcheckbox" = "menuitem") {
  const menu = await openMenu(origin, menuName);
  await fireEvent.click(within(menu).getByRole(role, { name: item }));
}

/** Opens the menu of `origin`, returns the mirror ids of its items and closes it again. */
export async function itemIds(origin: HTMLElement): Promise<string[]> {
  await fireEvent.contextMenu(origin);
  const menu = screen.getByRole("menu");
  const ids = menuItems(menu).map((item) => item.dataset.item ?? "");
  await fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Escape" });
  expect(screen.queryByRole("menu")).toBeNull();
  return ids;
}

/** True when a control marked with this mirror id is on the page, or behind a disclosure whose summary is. */
export function mirrored(id: string): boolean {
  return [...document.querySelectorAll<HTMLElement>(`[data-mirror="${id}"]`)].some((control) => {
    const closed = control.closest("details:not([open])");
    return !control.closest("[role='menu']") && (closed === null || closed.querySelector("summary") !== null);
  });
}
