import { fireEvent, screen, within } from "@testing-library/svelte";
import type { SetupStoreView } from "../lib/api/bindings/SetupStoreView";
import { effectHandlers } from "./effects";
import { setupDemoTower } from "./fixtures/setups";

type Handler = (args: Record<string, unknown>) => unknown;

/** A setup store that keeps what is saved, so a test can read back what the screen sent. */
export function setupHandlers(start: SetupStoreView = setupDemoTower, extra: Record<string, Handler> = {}): Record<string, Handler> {
  let stored = structuredClone(start);
  return {
    ...effectHandlers(),
    setup: () => stored,
    save_setup: (args) => {
      stored = { ...stored, setup: structuredClone(args.setup) as SetupStoreView["setup"] };
      return stored;
    },
    ...extra,
  };
}

export async function openSetup() {
  const rail = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(within(rail).getByRole("button", { name: "Setup" }));
  return screen.findByRole("listbox", { name: /^Components in the/ });
}

export function componentRow(list: HTMLElement, name: string) {
  return within(list).getByRole("option", { name }) as HTMLElement;
}

export function inspector() {
  return screen.getByRole("region", { name: /^Edit / });
}
