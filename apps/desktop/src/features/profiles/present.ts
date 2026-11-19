import type { ApplyReport } from "../../lib/api/bindings/ApplyReport";
import type { Change } from "../../lib/api/bindings/Change";
import type { DeviceRef } from "../../lib/api/bindings/DeviceRef";
import type { EffectProblem } from "../../lib/api/bindings/EffectProblem";
import type { ItemOutcome } from "../../lib/api/bindings/ItemOutcome";
import type { ConnectApply } from "../../lib/api/bindings/ConnectApply";
import type { MarkProblem } from "../../lib/api/bindings/MarkProblem";
import type { Marked } from "../../lib/api/bindings/Marked";
import type { OnConnect } from "../../lib/api/bindings/OnConnect";
import type { ProfileItem } from "../../lib/api/bindings/ProfileItem";
import type { ProfileSummary } from "../../lib/api/bindings/ProfileSummary";
import type { ResolutionCounts } from "../../lib/api/bindings/ResolutionCounts";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { SkipReason } from "../../lib/api/bindings/SkipReason";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
import { failureText, fileProblemText } from "../../lib/errors";
import type { Names } from "../../lib/names";
import { capitalise, serverName } from "../connection/wording";
import { reference } from "../devices/present";

/** What the texts need to name devices, zones and effects the way the rest of the app does. */
export type Labels = {
  names: Names;
  session: SessionSnapshot | null;
  effect: (id: string) => string | null;
};

export function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

export function contents(profile: ProfileSummary): string {
  return [
    count(profile.effects, "effect", "effects"),
    count(profile.modes, "hardware mode", "hardware modes"),
    count(profile.external, "device for another program", "devices for another program"),
  ].join(", ");
}

export function found(total: number, counts: ResolutionCounts | null): string {
  if (counts === null) {
    return "Devices not checked: PrismForge is not connected.";
  }
  if (counts.resolved === total) {
    return total === 1 ? "Its device is found." : `All ${total} devices found.`;
  }
  return `${counts.resolved} of ${total} devices found. ${total - counts.resolved} not found today.`;
}

export function deviceLabel(labels: Labels, device: DeviceRef | null, stored: string): string {
  return (device === null ? null : labels.names.given(device)) ?? stored;
}

function zoneLabel(labels: Labels, device: DeviceRef | null, zone: number, stored: string | null): string {
  const given = device === null ? null : labels.names.givenZone(device, zone);
  if (given) {
    return given;
  }
  if (stored) {
    return stored;
  }
  const current = labels.session?.devices.find((candidate) => reference(candidate) === device);
  return current?.descriptor.zones[zone]?.name ?? `zone ${zone + 1}`;
}

export function itemText(
  labels: Labels,
  item: ProfileItem,
  device: DeviceRef | null,
  stored: { zone?: string | null; effect?: string | null } = {},
): string {
  if ("Effect" in item) {
    const { zone, effect } = item.Effect;
    const name = stored.effect ?? labels.effect(effect) ?? `the effect “${effect}”`;
    const where = zone === null ? "the whole device" : zoneLabel(labels, device, zone, stored.zone ?? null);
    return `${name} on ${where}`;
  }
  if ("HardwareMode" in item) {
    return `Hardware mode ${item.HardwareMode.name}`;
  }
  return `Another program: ${item.External.label}`;
}

export function skipText(labels: Labels, reason: SkipReason): string {
  const server = labels.session ? serverName(labels.session) : "OpenRGB";
  if (reason === "Missing") {
    return "Not found. No connected device has this model and these zones.";
  }
  if (reason === "Ambiguous") {
    return "Identical devices. PrismForge cannot tell which one this is, so it uses none.";
  }
  if (reason === "Unavailable") {
    return `Not available. ${capitalise(server)} does not report this device right now.`;
  }
  if (reason === "NoDirectMode") {
    return "No Direct Mode. PrismForge effects cannot run on this device.";
  }
  if (reason === "SetupChanged") {
    return "The setup changed. The setup view this effect runs across is gone, or this device is no longer placed in it.";
  }
  if ("DeclaredExternal" in reason) {
    return `You said ${reason.DeclaredExternal.label} controls this device, so PrismForge sends it nothing.`;
  }
  if ("ModeListChanged" in reason) {
    const found = reason.ModeListChanged.found;
    return found
      ? `The device's mode list changed: that place now holds ${found}. PrismForge does not select another mode.`
      : "The device's mode list changed: that mode is gone. PrismForge does not select another mode.";
  }
  if ("NotAvailable" in reason) {
    return effectProblemText(reason.NotAvailable.problem);
  }
  return `Refused. ${failureText(reason.Refused.error)}`;
}

export function effectProblemText(problem: EffectProblem): string {
  if ("EffectNotAvailable" in problem) {
    return "Not available in this version: this PrismForge does not have this effect.";
  }
  if ("ControlsNotAvailable" in problem) {
    return "Not available in this version: some of its settings do not exist in this PrismForge.";
  }
  return "Not available in this version: some of its settings are not valid in this PrismForge.";
}

export function outcomeText(labels: Labels, outcome: ItemOutcome): string {
  return outcome === "Applied" ? "Applied." : `Skipped. ${skipText(labels, outcome.Skipped.reason)}`;
}

export function fullyApplied(report: ApplyReport): boolean {
  return "Applied" in report.frame_rate && report.items.length > 0 && report.items.every((item) => item.outcome === "Applied");
}

export function applySummary(report: ApplyReport): string {
  const total = report.items.length;
  const skipped = report.items.filter((item) => item.outcome !== "Applied").length;
  const rate = "Refused" in report.frame_rate ? " The frame rate was refused." : "";
  if (total === 0) {
    return `There was nothing to apply: it holds no effect, hardware mode or program statement.${rate}`;
  }
  if (skipped === 0) {
    return rate ? rate.trim() : `${total === 1 ? "Its setting was" : `All ${total} settings were`} accepted.`;
  }
  if (skipped === total) {
    const every = total === 1 ? "Its setting was skipped and stays" : `All ${total} settings were skipped and stay`;
    return `${every} in the profile.${rate}`;
  }
  return `${skipped} of ${total} settings were skipped and stay in the profile.${rate}`;
}

/** A partial result, a refused frame rate or an empty profile never reads as a complete apply. */
export function applyHeadline(name: string, report: ApplyReport): string {
  const total = report.items.length;
  const skipped = report.items.filter((item) => item.outcome !== "Applied").length;
  const rate = "Refused" in report.frame_rate ? " The frame rate was refused." : "";
  if (total === 0) {
    return `${name} has nothing to apply: it holds no effect, hardware mode or program statement.${rate}`;
  }
  if (fullyApplied(report)) {
    return `${name} is applied. ${applySummary(report)}`;
  }
  if (skipped === total) {
    return `Nothing from ${name} was applied. ${applySummary(report)}`;
  }
  return `${name} is applied in part. ${applySummary(report)}`;
}

export function connectHeadline(name: string, report: ApplyReport): string {
  const total = report.items.length;
  const skipped = report.items.filter((item) => item.outcome !== "Applied").length;
  if (fullyApplied(report)) {
    return `PrismForge applied ${name} when it connected. ${applySummary(report)}`;
  }
  if (total === 0 || skipped === total) {
    return `PrismForge connected. ${applyHeadline(name, report)}`;
  }
  return `PrismForge applied ${name} in part when it connected. ${applySummary(report)}`;
}

function stopped(session: SessionSnapshot | null): boolean {
  const connection = session?.connection;
  return typeof connection === "object" && connection !== null && "Disconnected" in connection && connection.Disconnected.reason !== null;
}

function markProblemText(problem: MarkProblem): string {
  return problem === "NewerVersion" ? "needs a newer PrismForge, so this version cannot read it" : "is not in the store any more";
}

function profileLabel(name: string | null): string {
  return name ?? "the profile marked to apply on connect";
}

/** The one text for what apply on connect did; the notice and the status line both use it. */
export function connectResultText(ran: ConnectApply): string {
  const outcome = ran.outcome;
  if ("Applied" in outcome) {
    return connectHeadline(profileLabel(ran.name), outcome.Applied.report);
  }
  return `PrismForge could not apply ${profileLabel(ran.name)} when it connected. ${failureText(outcome.Failed.error)}`;
}

function nextText(next: Marked | null): string {
  if (next === null) {
    return "";
  }
  return "Ready" in next
    ? ` After you disconnect and connect, PrismForge applies ${next.Ready.name} by itself.`
    : ` The profile marked for the next connect ${markProblemText(next.Unreadable.problem)}, so nothing is applied then.`;
}

/** Every sentence about a pending apply on connect comes from the state the shell publishes. */
export function onConnectText(state: OnConnect, session: SessionSnapshot | null): string {
  if (stopped(session)) {
    let next = "";
    if (state === "NotArmed") {
      next = " No profile is marked to apply on connect.";
    } else if (typeof state === "object" && "Armed" in state) {
      next = ` When you connect, PrismForge applies ${state.Armed.name} as soon as it has read the devices.`;
    } else if (typeof state === "object" && "Unreadable" in state) {
      next = ` The profile marked to apply on connect ${markProblemText(state.Unreadable.problem)}. Nothing is applied when you connect.`;
    }
    return `The connection stopped and PrismForge does not try again. Connect again on the Devices screen.${next}`;
  }
  if (state === "NotApplicable") {
    return session === null
      ? "When you connect, PrismForge applies the profile marked to apply on connect, if there is one, as soon as it has read the devices."
      : "Profiles are not available in this window, so no profile is applied when PrismForge connects.";
  }
  if (state === "NotArmed") {
    return "No profile is marked to apply on connect. A profile you mark before PrismForge has read the devices is applied then.";
  }
  if ("Armed" in state) {
    return `PrismForge will apply ${state.Armed.name} as soon as it has read the devices.`;
  }
  if ("Unreadable" in state) {
    return `The profile marked to apply on connect ${markProblemText(state.Unreadable.problem)}. Nothing is applied when PrismForge connects.`;
  }
  const { ran, next } = state.Fired;
  const done = ran
    ? `${connectResultText(ran)} Apply on connect does not run again by itself in this session, also not when the connection comes back. You can still apply a profile yourself.`
    : "No profile was marked when PrismForge connected, so none was applied by itself. A profile you mark now is not applied by itself in this session. You can still apply it yourself.";
  return `${done}${nextText(next)}`;
}

export function notConnectedText(state: OnConnect, session: SessionSnapshot | null): string {
  if (stopped(session)) {
    return onConnectText(state, session);
  }
  const lead = session === null ? "PrismForge is not connected. Connect first." : "PrismForge is not connected right now.";
  return `${lead} ${onConnectText(state, session)}`;
}

export function waitingText(name: string, state: OnConnect, session: SessionSnapshot | null): string {
  if (typeof state === "object" && "Armed" in state && state.Armed.name === name) {
    return `${name} will be applied as soon as PrismForge has read the devices.`;
  }
  return `${name} was not applied now: PrismForge is not connected. ${onConnectText(state, session)}`;
}

export function frameRateText(report: ApplyReport): string {
  const outcome = report.frame_rate;
  return "Applied" in outcome
    ? `Frame rate set to ${outcome.Applied.in_use} frames per second.`
    : `The frame rate was refused. ${failureText(outcome.Refused.error)}`;
}

export function changeText(labels: Labels, device: DeviceRef, change: Change): string {
  if ("Added" in change) {
    return `adds ${lower(itemText(labels, change.Added.now, device))}`;
  }
  if ("Replaced" in change) {
    const { was, now } = change.Replaced;
    return `replaces ${lower(itemText(labels, was, device))} with ${lower(itemText(labels, now, device))}`;
  }
  return `removes ${lower(itemText(labels, change.Removed.was, device))}`;
}

function lower(text: string): string {
  return text.startsWith("Hardware mode") || text.startsWith("Another program") ? text.charAt(0).toLowerCase() + text.slice(1) : text;
}

export function noticeText(notice: StoreNotice): string[] {
  const file = notice.file;
  const event = notice.event;
  if ("RestoredFromBackup" in event) {
    const { problem, set_aside } = event.RestoredFromBackup;
    return [
      problem === null
        ? `${file} was missing, so PrismForge used its backup.`
        : `PrismForge could not read ${file} and used its backup. ${fileProblemText(problem)}`,
      "Your last change to it may be missing. Check the profile or the device names.",
      ...(set_aside ? [`The damaged file is still there, now called ${set_aside}.`] : []),
    ];
  }
  if ("StartedEmpty" in event) {
    const { set_aside } = event.StartedEmpty;
    return [
      `PrismForge could not read ${file} or its backup, so it started that part empty.`,
      ...(set_aside.length > 0 ? [`The damaged files are still there: ${set_aside.join(", ")}.`] : []),
      "Close PrismForge before you touch them. If you have a copy of the file from elsewhere, put it back under the original name.",
    ];
  }
  if ("LeftUntouched" in event) {
    return [
      `${file} needs a newer PrismForge. This version does not change it.`,
      "Install the newer version, or keep using this version without it.",
    ];
  }
  const failure = event.WriteFailed.failure;
  return [`PrismForge could not save ${failure.file}: ${failure.detail}.`, "The previous version is still in place."];
}

export function unsavedText(unsaved: number | null): string {
  if (unsaved === null) {
    return "Unsaved changes unknown";
  }
  return unsaved === 1 ? "1 unsaved change" : `${unsaved} unsaved changes`;
}
