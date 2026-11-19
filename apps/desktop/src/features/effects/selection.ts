import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import { capitalise, serverName } from "../connection/wording";
import { hasDirectMode, reference } from "../devices/present";
import { declaredBy, sameTarget, targetLeds } from "./present";

/** One row of the target list: a whole device, or one zone of a device with more than one zone. */
export type TargetOption = { target: OutputTarget; device: SessionDevice; reason: string | null };

/** Why a target can never join a selection of several now; passing states such as an identify do not count. */
export function choiceBlocked(session: SessionSnapshot, device: SessionDevice, zone: number | null): string | null {
  const ref = reference(device);
  if (ref === null) {
    return "PrismForge cannot tell this device apart from an identical one.";
  }
  if (!device.available) {
    return `${capitalise(serverName(session))} does not report this device right now.`;
  }
  const label = declaredBy(session, ref);
  if (label !== null) {
    return `You said ${label} controls this device. Hand it back before you apply an effect.`;
  }
  if (!hasDirectMode(device)) {
    return "This device has no Direct Mode, so PrismForge effects cannot run on it.";
  }
  if (targetLeds(device, zone) === 0) {
    return "This zone has no LEDs.";
  }
  return null;
}

export function options(session: SessionSnapshot): TargetOption[] {
  return session.devices.flatMap((device) => {
    const ref = reference(device);
    if (ref === null) {
      return [];
    }
    const zones = device.descriptor.zones.length > 1 ? device.descriptor.zones.map((_, index) => index) : [];
    return [null, ...zones].map((zone) => ({ target: { device: ref, zone }, device, reason: choiceBlocked(session, device, zone) }));
  });
}

function zonesOf(session: SessionSnapshot, device: string): number[] {
  const found = session.devices.find((each) => reference(each) === device);
  const count = found?.descriptor.zones.length ?? 0;
  return count > 1 ? Array.from({ length: count }, (_, index) => index) : [];
}

export function choosable(session: SessionSnapshot, target: OutputTarget): boolean {
  return options(session).some((option) => sameTarget(option.target, target) && option.reason === null);
}

export function isSelected(selection: OutputTarget[], target: OutputTarget): boolean {
  return selection.some((each) => sameTarget(each, target) || (target.zone !== null && each.device === target.device && each.zone === null));
}

export function isMixed(selection: OutputTarget[], device: string): boolean {
  return selection.some((each) => each.device === device && each.zone !== null);
}

/** A whole device covers its zones, and a device with every zone selected becomes the whole device. */
function normalize(selection: OutputTarget[], session: SessionSnapshot): OutputTarget[] {
  const kept: OutputTarget[] = [];
  for (const target of selection) {
    if (kept.some((each) => sameTarget(each, target))) {
      continue;
    }
    if (target.zone !== null && selection.some((each) => each.device === target.device && each.zone === null)) {
      continue;
    }
    kept.push(target);
  }
  const whole = [...new Set(kept.map((target) => target.device))].filter((device) => {
    const zones = zonesOf(session, device);
    return zones.length > 0 && zones.every((zone) => kept.some((each) => each.device === device && each.zone === zone));
  });
  return [
    ...kept.filter((target) => !whole.includes(target.device)),
    ...whole.map((device) => ({ device, zone: null })),
  ];
}

export function toggle(selection: OutputTarget[], target: OutputTarget, session: SessionSnapshot): OutputTarget[] {
  if (!choosable(session, target)) {
    return selection;
  }
  if (target.zone === null) {
    const others = selection.filter((each) => each.device !== target.device);
    return isSelected(selection, target) ? others : [...others, target];
  }
  if (selection.some((each) => each.device === target.device && each.zone === null)) {
    const rest = zonesOf(session, target.device)
      .filter((zone) => zone !== target.zone && choosable(session, { device: target.device, zone }))
      .map((zone) => ({ device: target.device, zone }));
    return [...selection.filter((each) => each.device !== target.device), ...rest];
  }
  if (isSelected(selection, target)) {
    return selection.filter((each) => !sameTarget(each, target));
  }
  return normalize([...selection, target], session);
}

export function selectAll(session: SessionSnapshot): OutputTarget[] {
  return options(session)
    .filter((option) => option.target.zone === null && option.reason === null)
    .map((option) => option.target);
}

export function addRange(selection: OutputTarget[], from: OutputTarget, to: OutputTarget, session: SessionSnapshot): OutputTarget[] {
  const order = options(session);
  const start = order.findIndex((option) => sameTarget(option.target, from));
  const end = order.findIndex((option) => sameTarget(option.target, to));
  if (start < 0 || end < 0) {
    return selection;
  }
  const [low, high] = start <= end ? [start, end] : [end, start];
  const added = order.slice(low, high + 1).filter((option) => option.reason === null).map((option) => option.target);
  return normalize([...selection, ...added], session);
}

/** The targets a command goes to: blocked targets drop out of a selection of several; a single one stays so its page can say why. */
export function effective(selection: OutputTarget[], session: SessionSnapshot): OutputTarget[] {
  if (selection.length <= 1) {
    return selection;
  }
  const listed = selection.filter((target) => options(session).some((option) => sameTarget(option.target, target)));
  return listed.length <= 1 ? listed : listed.filter((target) => choosable(session, target));
}

export function countText(count: number): string {
  if (count === 0) {
    return "No target selected";
  }
  return count === 1 ? "1 target selected" : `${count} targets selected`;
}
