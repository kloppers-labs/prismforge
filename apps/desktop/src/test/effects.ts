import { fireEvent, screen, within } from "@testing-library/svelte";
import type { EffectInstance } from "../lib/api/bindings/EffectInstance";
import type { OutputCondition } from "../lib/api/bindings/OutputCondition";
import type { OutputOwner } from "../lib/api/bindings/OutputOwner";
import type { OutputTarget } from "../lib/api/bindings/OutputTarget";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
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
  const screens = await screen.findByRole("navigation", { name: "Main" });
  await fireEvent.click(await within(screens).findByRole("button", { name: "Effects" }));
  return screen.findByRole("navigation", { name: "Targets" });
}

export async function chooseTarget(device: string, part = "Whole device", index = 0) {
  const targets = await openEffectsIfNeeded();
  const group = within(targets).getAllByRole("group", { name: device })[index];
  if (!group) {
    throw new Error(`no target group ${device}`);
  }
  await fireEvent.click(within(group).getByRole("option", { name: new RegExp(`^${part}`) }));
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

export function withoutEffectsOn(session: SessionSnapshot, device: string): SessionSnapshot {
  const next = structuredClone(session);
  next.output.scene.assignments = next.output.scene.assignments.filter((assignment) => assignment.target.device !== device);
  next.output.devices = next.output.devices.filter((output) => output.device !== device);
  return next;
}

export function withOwner(session: SessionSnapshot, device: string, owner: OutputOwner): SessionSnapshot {
  const next = withoutEffectsOn(session, device);
  next.output.devices.push({ device, owner, condition: "Idle" });
  return next;
}

export function withEffect(session: SessionSnapshot, target: OutputTarget, effect: EffectInstance, condition: OutputCondition): SessionSnapshot {
  const next = withoutEffectsOn(session, target.device);
  next.output.scene.assignments.push({ target, effect });
  next.output.devices.push({ device: target.device, owner: "Effect", condition });
  return next;
}
