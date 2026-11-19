import type { Assignment } from "../../lib/api/bindings/Assignment";
import type { DeviceOutput } from "../../lib/api/bindings/DeviceOutput";
import type { OutputCondition } from "../../lib/api/bindings/OutputCondition";
import type { OutputOwner } from "../../lib/api/bindings/OutputOwner";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { RestoreOutcome } from "../../lib/api/bindings/RestoreOutcome";
import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../../lib/api/bindings/ShellView";
import { backendErrorText } from "../../lib/errors";
import type { Names } from "../../lib/names";
import { capitalise, serverName } from "../connection/wording";
import { restoreFailureText } from "../devices/outcome";
import { hasDirectMode, isLive, reference } from "../devices/present";

export function sameTarget(left: OutputTarget, right: OutputTarget): boolean {
  return left.device === right.device && left.zone === right.zone;
}

export function deviceOf(session: SessionSnapshot, target: OutputTarget): SessionDevice | null {
  return session.devices.find((device) => reference(device) === target.device) ?? null;
}

export function outputOf(session: SessionSnapshot, device: string): DeviceOutput | null {
  return session.output.devices.find((output) => output.device === device) ?? null;
}

/** The effect running on exactly this target. */
export function runningOn(session: SessionSnapshot, target: OutputTarget): Assignment | null {
  return session.output.scene.assignments.find((assignment) => sameTarget(assignment.target, target)) ?? null;
}

/** True when stopping `target` would stop at least one effect; a whole device covers its zones. */
export function canStop(session: SessionSnapshot, target: OutputTarget): boolean {
  return session.output.scene.assignments.some(
    (assignment) =>
      assignment.target.device === target.device && (target.zone === null || assignment.target.zone === target.zone),
  );
}

export function targetName(names: Names, device: SessionDevice, zone: number | null): string {
  return zone === null ? names.device(device) : names.zone(device, zone);
}

/** Why no output command can be sent now: the shell refuses them all while a profile applies. */
export function applyingText(view: ShellView): string | null {
  return view.applying ? `PrismForge is applying the profile ${view.applying.name}. Wait until it finishes.` : null;
}

export function targetLeds(device: SessionDevice, zone: number | null): number {
  return zone === null ? device.descriptor.led_count : (device.descriptor.zones[zone]?.led_count ?? 0);
}

export function modeName(device: SessionDevice, mode: number): string {
  return device.descriptor.capabilities.modes[mode]?.name ?? `mode ${mode + 1}`;
}

function externalLabel(owner: OutputOwner | undefined): string | null {
  return owner && typeof owner === "object" && "External" in owner ? owner.External.label : null;
}

export function declaredBy(session: SessionSnapshot, device: string): string | null {
  return externalLabel(outputOf(session, device)?.owner);
}

function identifying(session: SessionSnapshot, device: string): boolean {
  return session.identify?.device === device;
}

/** Why no effect can be applied to this target now, or null. */
export function applyBlocked(session: SessionSnapshot, device: SessionDevice, zone: number | null): string | null {
  const ref = reference(device);
  if (ref === null) {
    return "PrismForge cannot tell this device apart from an identical one.";
  }
  if (!isLive(session)) {
    return "PrismForge is not connected. Effects can start again after it reconnects.";
  }
  if (!device.available) {
    return `${capitalise(serverName(session))} does not report this device right now.`;
  }
  const label = declaredBy(session, ref);
  if (label !== null) {
    return `You said ${label} controls this device. Hand it back before you apply an effect.`;
  }
  if (!hasDirectMode(device)) {
    return "This device has no Direct Mode, so PrismForge effects cannot run on it. Its hardware modes still work.";
  }
  if (identifying(session, ref)) {
    return "An identify is running on this device. Wait until it finishes.";
  }
  if (targetLeds(device, zone) === 0) {
    return "This zone has no LEDs.";
  }
  return null;
}

export function modeBlocked(session: SessionSnapshot, device: SessionDevice): string | null {
  const ref = reference(device);
  if (ref === null) {
    return "PrismForge cannot tell this device apart from an identical one.";
  }
  if (!isLive(session)) {
    return "PrismForge is not connected.";
  }
  if (!device.available) {
    return `${capitalise(serverName(session))} does not report this device right now.`;
  }
  const label = declaredBy(session, ref);
  if (label !== null) {
    return `You said ${label} controls this device. Hand it back first.`;
  }
  if (identifying(session, ref)) {
    return "An identify is running on this device. Wait until it finishes.";
  }
  return null;
}

export function stopBlocked(session: SessionSnapshot, target: OutputTarget): string | null {
  if (identifying(session, target.device)) {
    return "An identify is running on this device. Wait until it finishes.";
  }
  return canStop(session, target) ? null : "No PrismForge effect runs on this target.";
}

export type TargetMark = { text: string; tone: "good" | "caution" | "plain" };

export function targetMark(session: SessionSnapshot, device: SessionDevice): TargetMark | null {
  const ref = reference(device);
  const output = ref === null ? null : outputOf(session, ref);
  if (!output) {
    return null;
  }
  const owner = output.owner;
  if (owner === "Effect") {
    const state = shortCondition(output.condition);
    return { text: `Effect ${state}`, tone: state === "running" ? "good" : state === "idle" ? "plain" : "caution" };
  }
  if ("HardwareMode" in owner) {
    return { text: `Hardware mode ${modeName(device, owner.HardwareMode.mode)}`, tone: "plain" };
  }
  return { text: `Left to ${owner.External.label}`, tone: "caution" };
}

function shortCondition(condition: OutputCondition): string {
  if (condition === "Running" || condition === "Idle") {
    return condition === "Running" ? "running" : "idle";
  }
  return "Paused" in condition ? "paused" : "refused";
}

export function ownerText(session: SessionSnapshot, device: SessionDevice, output: DeviceOutput | null): string {
  if (!output) {
    return "None. PrismForge sends nothing to this device.";
  }
  const owner = output.owner;
  if (owner === "Effect") {
    return session.backend.origin === "Simulated"
      ? "A PrismForge effect, computed by PrismForge and sent to the demo backend. Nothing reaches hardware."
      : "A PrismForge effect, computed by PrismForge and sent to the device through OpenRGB.";
  }
  if ("HardwareMode" in owner) {
    return `Hardware mode ${modeName(device, owner.HardwareMode.mode)}, selected in PrismForge. It runs on the device itself. PrismForge writes nothing to this device.`;
  }
  return `${capitalise(owner.External.label)}, as you said. PrismForge cannot see that program. It sends nothing to this device.`;
}

export function conditionText(session: SessionSnapshot, output: DeviceOutput | null): string | null {
  if (!output || output.condition === "Idle") {
    return null;
  }
  const condition = output.condition;
  if (condition === "Running") {
    return "Running.";
  }
  if ("Refused" in condition) {
    return `Refused. ${capitalise(serverName(session))} refused the last frame. PrismForge tries again after a wait that grows with each refusal.`;
  }
  const reason = condition.Paused.reason;
  if (reason === "NotConnected") {
    return "Paused. PrismForge is not connected. The effect resumes after it reconnects.";
  }
  if (reason === "InventoryStale") {
    return "Paused. The device list is being read again. The effect resumes after the scan.";
  }
  if (reason === "Identify") {
    return "Paused. An identify is running on this device. The effect resumes after it.";
  }
  return `Paused. ${backendErrorText(reason.Unavailable.error)} The effect resumes when the device is back as it was.`;
}

export type StopText = { settled: boolean; headline: string; details: string[] };

export function stopText(outcome: RestoreOutcome | null, server: string): StopText {
  if (outcome === null) {
    return {
      settled: true,
      headline: "Effect stopped. Other PrismForge effects still run on this device, so its earlier lighting was not put back yet.",
      details: [],
    };
  }
  if (outcome === "Unchanged") {
    return { settled: true, headline: "Effect stopped. Nothing was changed on the device.", details: [] };
  }
  if (outcome === "Restored") {
    return {
      settled: true,
      headline: `Effect stopped. ${capitalise(server)} accepted the requests to put back the lighting from before the effect.`,
      details: ["PrismForge cannot confirm what the device shows now."],
    };
  }
  if ("PartlyRestored" in outcome) {
    const failure = outcome.PartlyRestored.failure;
    if (failure === "PreviousStateUnknown") {
      return {
        settled: false,
        headline: "Effect stopped, but the earlier lighting may not be back.",
        details: [
          "PrismForge did not know the earlier lighting of this device. It put back what it had.",
          "Check the device, and select a hardware mode below if needed.",
        ],
      };
    }
    return {
      settled: false,
      headline: "Effect stopped. Only part of the earlier lighting was put back.",
      details: [restoreFailureText(failure, server), "Select a hardware mode below if the device looks wrong."],
    };
  }
  return {
    settled: false,
    headline: "Effect stopped. The earlier lighting was not put back.",
    details: [restoreFailureText(outcome.NotRestored.failure, server), "Select a hardware mode below, or set the lighting again in OpenRGB."],
  };
}

export function controlSummary(session: SessionSnapshot, device: SessionDevice, effectLabel: (id: string) => string | null): string | null {
  const ref = reference(device);
  const output = ref === null ? null : outputOf(session, ref);
  if (!output || ref === null) {
    return null;
  }
  const owner = output.owner;
  if (owner === "Effect") {
    const effects = session.output.scene.assignments.filter((assignment) => assignment.target.device === ref);
    const names = [...new Set(effects.map((assignment) => effectLabel(assignment.effect.effect) ?? assignment.effect.effect))];
    const what = names.length === 1 ? `the PrismForge effect ${names[0]}` : "PrismForge effects";
    const state = shortCondition(output.condition);
    return state === "running" || state === "idle" ? `Runs ${what}.` : `Runs ${what}, ${state} now.`;
  }
  if ("HardwareMode" in owner) {
    return `Runs hardware mode ${modeName(device, owner.HardwareMode.mode)}, selected in PrismForge.`;
  }
  return `Left to ${owner.External.label}. PrismForge sends nothing to it.`;
}

export type Mode = { name: "Live" | "Draft" | "Off"; text: string };

export type Nearby = { label: string; where: string | null };

function nearbyText(nearby: Nearby[], chosen: string | null): string {
  const one = nearby.length === 1 ? nearby[0] : null;
  if (one && one.where === null) {
    return `${one.label} runs on the whole device. Apply effect replaces it.`;
  }
  const what = one
    ? `${one.label} runs on ${one.where}.`
    : `Effects run on ${nearby.length} zones: ${nearby.map((each) => `${each.where} (${each.label})`).join(", ")}.`;
  const it = one ? "it" : "them";
  const replace = chosen === null ? "" : ` Apply effect replaces ${it} with one effect on the whole device.`;
  return `${what}${replace} Stop effect stops ${it}.`;
}

export function editMode(state: {
  blocked: string | null;
  live: boolean;
  chosen: string | null;
  running: string | null;
  runningLabel: string | null;
  nearby: Nearby[];
  target: string;
}): Mode {
  if (state.blocked !== null) {
    return { name: "Off", text: `Effects are off. ${state.blocked}` };
  }
  if (state.live) {
    return { name: "Live", text: `Changes go to ${state.target} as you make them.` };
  }
  if (state.running === null && state.nearby.length > 0) {
    return { name: "Draft", text: nearbyText(state.nearby, state.chosen) };
  }
  if (state.chosen === null) {
    return { name: "Draft", text: "Choose an effect. Your devices stay as they are until you apply it." };
  }
  if (state.running === state.chosen) {
    return { name: "Draft", text: "This effect runs now. Press Apply effect to send your changes." };
  }
  if (state.runningLabel !== null) {
    return { name: "Draft", text: `${state.runningLabel} runs now. Apply effect replaces it.` };
  }
  return { name: "Draft", text: "Your devices stay as they are until you press Apply effect." };
}

/** Effects on the same device that share LEDs with `target` without being on exactly `target`. */
export function nearbyEffects(session: SessionSnapshot, target: OutputTarget): Assignment[] {
  return session.output.scene.assignments.filter((assignment) => assignment.target.device === target.device && !sameTarget(assignment.target, target) && (target.zone === null || assignment.target.zone === null));
}
