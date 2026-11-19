import type { EffectDefinition } from "../../lib/api/bindings/EffectDefinition";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Names } from "../../lib/names";
import type { Menu, MenuItem } from "../../lib/shell/menu.svelte";
import { canStop, declaredBy, stopBlocked, targetName } from "./present";
import type { EffectsState } from "./state.svelte";

export type Reveal = "modes" | "external";

export type EffectsMenuContext = {
  effects: EffectsState;
  session: SessionSnapshot;
  names: Names;
  held: string | null;
  reveal: (part: Reveal) => Promise<void>;
};

const WORKING = "PrismForge is still working on an earlier request.";

function onScreen(effects: EffectsState, target: OutputTarget): boolean {
  return effects.target !== null && effects.target.device === target.device && effects.target.zone === target.zone;
}

export function targetMenu(context: EffectsMenuContext, device: SessionDevice, target: OutputTarget): Menu {
  const { effects, session, held } = context;
  const show = () => onScreen(effects, target) || effects.select(target);
  const left = declaredBy(session, target.device);
  const items: MenuItem[] = [{ id: "target.choose", label: "Select only this target", run: () => effects.select(target) }];
  if (canStop(session, target)) {
    items.push({
      id: "target.stop",
      label: "Stop effect",
      reason: held ?? stopBlocked(session, target) ?? (effects.working === null ? null : WORKING),
      run: () => (show(), effects.stop()),
    });
  }
  if (left !== null) {
    items.push({ id: "target.hand-back", label: "Hand back to PrismForge", reason: held, run: () => (show(), effects.clearExternal(target.device)) });
  }
  items.push({ id: "target.modes", label: "Hardware modes…", run: () => (show(), context.reveal("modes")) });
  if (left === null) {
    items.push({
      id: "target.leave",
      label: "Leave this device to another program…",
      danger: true,
      run: () => (show(), context.reveal("external")),
    });
  }
  return { label: `Actions for ${targetName(context.names, device, target.zone)}`, items };
}

export function effectMenu(context: EffectsMenuContext, definition: EffectDefinition, device: SessionDevice | null, blocked: string | null): Menu | null {
  const { effects } = context;
  const target = effects.target;
  if (effects.targets.length === 0 || (target !== null && declaredBy(context.session, target.device) !== null)) {
    return null;
  }
  const where = target !== null && device !== null ? targetName(context.names, device, target.zone) : `${effects.targets.length} targets`;
  const chosen = effects.effect === definition.id;
  const choose = () => chosen || effects.choose(definition.id);
  const unfinished = chosen && !effects.checked?.instance ? "Some settings need a fix first." : null;
  return {
    label: `Actions for ${definition.label}`,
    items: [
      { id: "effect.choose", label: "Choose this effect", run: choose },
      {
        id: "effect.apply",
        label: `Apply to ${where}`,
        reason: blocked ?? (effects.working === null ? null : WORKING) ?? unfinished,
        run: () => (choose(), effects.apply()),
      },
    ],
  };
}
