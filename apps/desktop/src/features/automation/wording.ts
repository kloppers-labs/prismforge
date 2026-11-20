import type { LightsOffOutcome } from "../../lib/api/bindings/LightsOffOutcome";
import type { LightsOffSkip } from "../../lib/api/bindings/LightsOffSkip";
import type { OpenRgbAct } from "../../lib/api/bindings/OpenRgbAct";
import type { OpenRgbSetting } from "../../lib/api/bindings/OpenRgbSetting";
import type { RuleAction } from "../../lib/api/bindings/RuleAction";
import type { RuleRun } from "../../lib/api/bindings/RuleRun";
import type { RuleView } from "../../lib/api/bindings/RuleView";
import type { RunFailure } from "../../lib/api/bindings/RunFailure";
import type { Trigger } from "../../lib/api/bindings/Trigger";
import type { EventSupport } from "../../lib/api/bindings/EventSupport";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
import { sessionErrorText, storeErrorText } from "../../lib/errors";
import { applyHeadline, count } from "../profiles/present";

export type ActionKind = "Nothing" | "LightsOff" | "ApplyProfile" | "Newer";

export const EVENTS: Record<Trigger, string> = {
  Starts: "PrismForge starts",
  Connects: "PrismForge connects",
  Sleeps: "the system goes to sleep",
  Wakes: "the system wakes",
  SessionEnds: "you sign out or the system shuts down or restarts",
  Quits: "PrismForge quits",
};

export const NOTE = "Rules run only while PrismForge runs.";

export const LIGHTS_OFF =
  "Switching the lights off writes black through Direct Mode, or selects a device's own “Off” mode when that mode does not save to device memory. Other devices are left as they are, and the last run names them.";

export function isSystemEvent(trigger: Trigger): boolean {
  return trigger === "Sleeps" || trigger === "Wakes" || trigger === "SessionEnds";
}

export function actionKind(action: RuleAction): ActionKind {
  return typeof action === "string" ? action : "ApplyProfile";
}

export function profileOf(action: RuleAction): string | null {
  return typeof action === "object" ? action.ApplyProfile.profile : null;
}

function actionText(rule: RuleView): string {
  const kind = actionKind(rule.action);
  if (kind === "Nothing") {
    return "do nothing";
  }
  if (kind === "LightsOff") {
    return "switch the lights off";
  }
  if (kind === "Newer") {
    return "do what a newer PrismForge set";
  }
  const profile = rule.profile;
  if (profile === null || profile === "Missing") {
    return "apply a profile that was deleted";
  }
  if (profile === "NeedsNewer") {
    return "apply a profile that needs a newer PrismForge";
  }
  return `apply the profile ${profile.Ready.name}`;
}

export function eventLabel(trigger: Trigger): string {
  return `When ${EVENTS[trigger]}`;
}

/** "When <event>, <action>." */
export function sentence(rule: RuleView): string {
  return `${eventLabel(rule.trigger)}, ${actionText(rule)}.`;
}

function verb(kind: ActionKind): { infinitive: string; present: string } {
  if (kind === "LightsOff") {
    return { infinitive: "switch the lights off", present: "switches the lights off" };
  }
  if (kind === "ApplyProfile") {
    return { infinitive: "apply the profile", present: "applies the profile" };
  }
  return { infinitive: "act", present: "acts" };
}

/** The one-line promise under each rule; `promiseDetailText` holds the rest. */
export function shortPromiseText(trigger: Trigger, action: RuleAction): string {
  const { infinitive, present } = verb(actionKind(action));
  switch (trigger) {
    case "Starts":
      return "PrismForge will do this once, after it starts and connects by itself.";
    case "Connects":
      return "PrismForge will do this once each time you connect.";
    case "Sleeps":
      return `PrismForge tries to ${infinitive} before the PC sleeps.`;
    case "Wakes":
      return `PrismForge ${present} once devices return, within about 30 seconds.`;
    case "SessionEnds":
      return `PrismForge tries to ${infinitive} as the session ends.`;
    case "Quits":
      return `PrismForge will ${infinitive} when you quit it.`;
  }
}

/** What the short promise leaves out: "will" where PrismForge decides, "tries to" where the system does. */
export function promiseDetailText(trigger: Trigger, action: RuleAction): string {
  switch (trigger) {
    case "Starts":
      return "That needs “Connect to OpenRGB when PrismForge starts” in Settings. At that connect this rule takes the place of the rule for “PrismForge connects”.";
    case "Connects":
      return "It acts as soon as the devices are read. A connection that PrismForge restores by itself after a loss does not count.";
    case "Sleeps":
      return "Sleep and hibernate are one event in Windows. Windows gives about two seconds and may skip the notice on low battery; PrismForge gives up after one second.";
    case "Wakes":
      return `It waits for OpenRGB and for the device list to stop changing, and uses the devices that are back by then.${actionKind(action) === "ApplyProfile" ? " It sets every mode and colour again." : ""}`;
    case "SessionEnds":
      return "Windows closes programs one by one and can end PrismForge or OpenRGB before the change reaches your devices. Devices that keep power after a shutdown may stay lit.";
    case "Quits":
      return "It does this before it ends, and waits at most two seconds.";
  }
}

/** A profile applied as PrismForge ends keeps its last frame; PrismForge no longer runs its effects. */
export function endingProfileText(trigger: Trigger, action: RuleAction): string | null {
  const ending = trigger === "Quits" || trigger === "SessionEnds";
  return ending && actionKind(action) === "ApplyProfile"
    ? "The profile's effects stop moving when PrismForge ends; each device keeps the frame it showed last."
    : null;
}

export function profileProblemText(rule: RuleView): string | null {
  if (actionKind(rule.action) !== "ApplyProfile") {
    return null;
  }
  if (rule.profile === null || rule.profile === "Missing") {
    return "The profile of this rule was deleted, so the rule applies nothing. Choose another profile or switch the rule off.";
  }
  if (rule.profile === "NeedsNewer") {
    return "The profile of this rule needs a newer PrismForge, so the rule applies nothing.";
  }
  return null;
}

const SETTING_WHEN: Record<OpenRgbSetting, string> = {
  Open: "when OpenRGB opens",
  ServiceStartup: "when the OpenRGB service starts",
  Suspend: "when the system goes to sleep",
  Resume: "when the system wakes",
  Exit: "when OpenRGB closes",
  ServiceShutdown: "when the OpenRGB service stops",
};

export function openRgbAlsoText(act: OpenRgbAct): string {
  return `OpenRGB also loads “${act.profile}” ${SETTING_WHEN[act.setting]}; whichever writes last wins.`;
}

export const OPENRGB_CHOOSE = "Turn PrismForge's rule or OpenRGB's profile setting off to choose which one acts.";

const SKIPS: Record<string, string> = {
  NotResolved: "PrismForge cannot tell it apart from an identical device",
  NotAvailable: "it was not there",
  NoDirectOrOffMode: "it has no Direct Mode and no “Off” mode that leaves its memory alone",
  OffModeSavesToDevice: "its “Off” mode saves to device memory, and frequent writes there wear it",
  Identifying: "it was being identified",
  TooLate: "the event was over before PrismForge came to it",
};

function skipText(reason: LightsOffSkip): string {
  if (typeof reason === "object") {
    return `you said ${reason.DeclaredExternal.label} controls it`;
  }
  return SKIPS[reason] ?? "PrismForge left it as it was";
}

function done(outcome: LightsOffOutcome): boolean {
  return outcome === "Black" || (typeof outcome === "object" && "OffMode" in outcome);
}

function seconds(ms: number): number {
  return Math.round(ms / 1000);
}

export function failureText(failure: RunFailure): string {
  if (failure === "Busy") {
    return "Did not run: PrismForge was applying a profile for you.";
  }
  if (failure === "NotConnected") {
    return "Did not run: PrismForge was not connected.";
  }
  if (failure === "TooLate") {
    return "Did not run: the event was over before the rule could write, so nothing was written.";
  }
  if ("StoreUnavailable" in failure) {
    return `No rule ran: PrismForge could not open its rules. ${failure.StoreUnavailable.detail}`;
  }
  if ("ProfileMissing" in failure) {
    return "Nothing was applied: the profile of this rule was deleted.";
  }
  if ("ProfileNeedsNewer" in failure) {
    return "Nothing was applied: the profile needs a newer PrismForge.";
  }
  if ("TimedOut" in failure) {
    return `Stopped after ${seconds(failure.TimedOut.limit_ms)} s, the time this event allows. Some devices may not have changed.`;
  }
  if ("OpenRgbNotBack" in failure) {
    return `Did not run: OpenRGB did not answer within ${seconds(failure.OpenRgbNotBack.waited_ms)} s after the wake.`;
  }
  if ("Store" in failure) {
    return `Did not run. ${storeErrorText(failure.Store.error)}`;
  }
  const error = failure.Output.error;
  return `Did not run. ${"Session" in error ? sessionErrorText(error.Session) : "PrismForge refused the change."}`;
}

export function runTime(run: RuleRun): string {
  return new Date(run.at_ms).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export type RunSummary = { headline: string; problems: string[]; leftAlone: string[] };

/** A device the user gave to another program, or one that was not there, is not a problem. */
function leftAlone(reason: LightsOffSkip): boolean {
  return reason === "NotAvailable" || typeof reason === "object";
}

/** The headline, the lines that need attention, and the devices left as they are on purpose. */
export function runSummary(run: RuleRun): RunSummary {
  const outcome = run.outcome;
  if ("Failed" in outcome) {
    return { headline: failureText(outcome.Failed.failure), problems: [], leftAlone: [] };
  }
  if ("Applied" in outcome) {
    const { name, report, missing, not_kept } = outcome.Applied;
    const problems: string[] = [];
    if (missing.length > 0) {
      problems.push(`Not back after the wake, so not changed: ${missing.join(", ")}.`);
    }
    if (not_kept) {
      const detail = "Session" in not_kept ? sessionErrorText(not_kept.Session) : "PrismForge refused the change.";
      problems.push(`PrismForge could not keep the profile's last frame as it ended, so the stop may have put back the earlier lighting. ${detail}`);
    }
    return { headline: applyHeadline(name, report), problems, leftAlone: [] };
  }
  const devices = outcome.LightsOff.report.devices;
  const off = devices.filter((device) => done(device.outcome)).length;
  const summary: RunSummary = { headline: `Switched the lights off on ${count(off, "device", "devices")}.`, problems: [], leftAlone: [] };
  for (const device of devices) {
    const result = device.outcome;
    if (typeof result === "object" && "Skipped" in result) {
      const line = `Skipped ${device.name}: ${skipText(result.Skipped.reason)}.`;
      (leftAlone(result.Skipped.reason) ? summary.leftAlone : summary.problems).push(line);
    } else if (typeof result === "object" && "Failed" in result) {
      const error = result.Failed.error;
      const detail = "Session" in error ? sessionErrorText(error.Session) : "PrismForge refused the change.";
      summary.problems.push(`Not switched off: ${device.name}. ${detail}`);
    }
  }
  return summary;
}

export function leftAloneText(devices: number): string {
  return `${count(devices, "device", "devices")} left as they are, named in Details.`;
}

/** Skips the session made for a step PrismForge had planned, as opposed to devices left out of the plan. */
function sessionSkip(outcome: LightsOffOutcome): LightsOffSkip | null {
  if (typeof outcome !== "object" || !("Skipped" in outcome)) {
    return null;
  }
  const reason = outcome.Skipped.reason;
  return reason === "TooLate" || reason === "Identifying" || typeof reason === "object" ? reason : null;
}

/** A run that did not do what the rule says: a failure, a frame not kept, a write too late, or no planned device switched off. */
export function runFailed(run: RuleRun): boolean {
  const outcome = run.outcome;
  if ("Failed" in outcome) {
    return true;
  }
  if ("Applied" in outcome) {
    return outcome.Applied.not_kept !== null;
  }
  const results = outcome.LightsOff.report.devices.map((device) => device.outcome);
  const failed = results.some((result) => typeof result === "object" && "Failed" in result);
  const late = results.some((result) => sessionSkip(result) === "TooLate");
  const planned = results.filter((result) => done(result) || sessionSkip(result) !== null || (typeof result === "object" && "Failed" in result)).length;
  const off = results.filter(done).length;
  return failed || late || (planned > 0 && off === 0);
}

const PAST: Partial<Record<Trigger, string>> = {
  Sleeps: "When the system went to sleep",
  SessionEnds: "When you signed out or the system shut down or restarted",
};

/** The notice at start for a failed sleep or session-end run the user has not seen. */
export function failedRunTitle(run: RuleRun): string {
  return `${PAST[run.trigger] ?? `When ${EVENTS[run.trigger]}`}, the rule did not do what it should: ${runSummary(run).headline}`;
}

export function supportText(support: EventSupport): string | null {
  if (support === "Available") {
    return null;
  }
  if (support === "NotOnThisSystem") {
    return "Works on Windows only for now.";
  }
  if ("Failed" in support) {
    return `PrismForge could not start receiving sleep, wake and session end, so those rules do nothing until PrismForge starts again: ${support.Failed.detail}`;
  }
  return `Part of PrismForge's set-up for system events failed. ${support.Limited.detail}`;
}

export function systemEventsBlocked(support: EventSupport): boolean {
  return support === "NotOnThisSystem" || (typeof support === "object" && "Failed" in support);
}

export function runNoticeText(notice: StoreNotice): string {
  const event = notice.event;
  if ("WriteFailed" in event) {
    return `PrismForge could not save ${event.WriteFailed.failure.file}: ${event.WriteFailed.failure.detail}. The last runs below are shown until PrismForge ends.`;
  }
  if ("LeftUntouched" in event) {
    return `${notice.file} needs a newer PrismForge, so this version does not record runs.`;
  }
  return `PrismForge could not read ${notice.file}, so earlier runs are not shown. The damaged file was set aside.`;
}
