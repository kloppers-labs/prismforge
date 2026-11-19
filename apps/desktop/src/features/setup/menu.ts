import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { Menu, MenuItem } from "../../lib/shell/menu.svelte";
import { nameOf, selectionName } from "./present";
import type { SetupState } from "./state.svelte";
import { outputOf } from "./wiring";

export type SetupMenuContext = { setup: SetupState; identify: (id: ComponentId) => void };

/** Acts on the selection when the row is part of several selected, otherwise selects only that row first, as a click would. */
export function componentMenu(context: SetupMenuContext, id: ComponentId): Menu {
  const { setup } = context;
  const together = setup.selection.length > 1 && setup.selection.includes(id);
  const focus = () => together || setup.select([id]);
  const ids = () => (together ? setup.selection : [id]);
  const viewName = setup.view?.name ?? "";
  const wired = (together ? setup.selection : [id]).some((each) => outputOf(setup.draft, each) !== null);
  const items: MenuItem[] = [
    { id: "component.select", label: "Select only this component", run: () => setup.select([id]) },
    { id: "component.turn", label: "Turn right 15°", run: () => (focus(), setup.turnSelection(1)) },
    { id: "component.flip", label: together ? "Flip each" : "Flip", run: () => (focus(), setup.flipSelection()) },
  ];
  if (!together) {
    items.push({
      id: "component.identify",
      label: "Identify",
      reason: outputOf(setup.draft, id) ? null : "Wire the component first.",
      run: () => (focus(), context.identify(id)),
    });
  }
  items.push({
    id: "component.group",
    label: "Make a group from the selection…",
    reason: wired ? null : "None of these components is wired, so a group would have no member.",
    run: (origin) => (focus(), (setup.asked = { kind: "group", from: origin })),
  });
  items.push({ id: "component.take-out", label: `Take out of the ${viewName} view`, run: () => setup.takeOut(ids()) });
  if (!together) {
    items.push({ id: "component.delete", label: "Delete component…", danger: true, run: (origin) => (focus(), (setup.asked = { kind: "delete", from: origin })) });
  }
  return { label: `Actions for ${together ? selectionName(setup.draft, setup.selection) : nameOf(setup.draft, id)}`, items };
}
