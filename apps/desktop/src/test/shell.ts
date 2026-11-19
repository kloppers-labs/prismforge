import { expect } from "vitest";
import { fireEvent, screen, waitFor, within } from "@testing-library/svelte";

export function rail() {
  return screen.getByRole("navigation", { name: "Main" });
}

export async function findRail() {
  return screen.findByRole("navigation", { name: "Main" });
}

export function railItems(): string[] {
  return within(rail())
    .getAllByRole("button")
    .map((item) => item.textContent?.replace(/\s+/g, " ").trim() ?? "");
}

export function railItem(name: string) {
  return within(rail()).getByRole("button", { name }) as HTMLButtonElement;
}

export function current(): string[] {
  return within(rail())
    .getAllByRole("button")
    .filter((item) => item.getAttribute("aria-current") === "page")
    .map((item) => item.textContent?.trim() ?? "");
}

export async function go(area: string) {
  await fireEvent.click(within(await findRail()).getByRole("button", { name: area }));
}

export function strip() {
  return screen.getByRole("region", { name: "Status" });
}

export async function findStrip() {
  return screen.findByRole("region", { name: "Status" });
}

export function connectionChip() {
  return within(strip()).getByRole("button", { name: /^Connection:/ }) as HTMLButtonElement;
}

export function profileChip() {
  return within(strip()).getByRole("button", { name: /^Profile:/ }) as HTMLButtonElement;
}

export async function openConnection() {
  await findStrip();
  await fireEvent.click(connectionChip());
  return screen.getByRole("group", { name: "Connection details" });
}

export async function openProfileChip() {
  await findStrip();
  await fireEvent.click(await within(strip()).findByRole("button", { name: /^Profile:/ }));
  return screen.getByRole("group", { name: "Profile details" });
}

export async function connectionAction(name: string | RegExp) {
  const panel = await openConnection();
  await fireEvent.click(within(panel).getByRole("button", { name }));
}

export function notices() {
  return screen.queryByRole("region", { name: "Notices" });
}

export async function waitForConnection(label: string) {
  await findStrip();
  await waitFor(() => expect(within(connectionChip()).getByText(label)).toBeTruthy());
}

export async function findDemoMarker() {
  const marker = await within(await findStrip()).findByText("Demo mode");
  return marker.closest("[data-demo]") as HTMLElement;
}

export async function expandNotices() {
  const toggle = screen.queryByRole("button", { name: /^Show all \d+ notices$/ });
  if (toggle) {
    await fireEvent.click(toggle);
  }
}
