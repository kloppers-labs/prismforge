import { fireEvent, screen, within } from "@testing-library/svelte";
import { catalog } from "./fixtures/catalog";
import { solidPreview } from "./fixtures/previews";

type Handler = (args: Record<string, unknown>) => unknown;

export function effectHandlers(extra: Record<string, Handler> = {}): Record<string, Handler> {
  return {
    effect_catalog: () => catalog,
    preview_effect: () => solidPreview,
    ...extra,
  };
}

export async function openEffects() {
  const screens = await screen.findByRole("navigation", { name: "Screens" });
  await fireEvent.click(within(screens).getByRole("button", { name: "Effects" }));
  return screen.findByRole("navigation", { name: "Targets" });
}

export async function chooseTarget(device: string, part = "Whole device", index = 0) {
  const targets = await openEffectsIfNeeded();
  const group = within(targets).getAllByRole("group", { name: device })[index];
  if (!group) {
    throw new Error(`no target group ${device}`);
  }
  await fireEvent.click(within(group).getByRole("button", { name: new RegExp(`^${part}`) }));
  await screen.findByRole("heading", { level: 2, name: new RegExp(`^${device}`) });
}

async function openEffectsIfNeeded() {
  return screen.queryByRole("navigation", { name: "Targets" }) ?? openEffects();
}

export function button(name: string | RegExp) {
  return screen.getByRole("button", { name }) as HTMLButtonElement;
}

export function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
