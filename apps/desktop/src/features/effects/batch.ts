import type { OutputError } from "../../lib/api/bindings/OutputError";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { TargetResult } from "../../lib/api/bindings/TargetResult";
import { failureText } from "../../lib/errors";
import type { Names } from "../../lib/names";
import { distinction } from "../devices/present";
import { retryable } from "./retry";
import { deviceOf, runningOn, stopText, targetName } from "./present";
import type { BatchFeedback } from "./state.svelte";

export type RowState = "Applied" | "Stopped" | "Skipped" | "Failed" | "Covered";

export type BatchRow = { key: string; label: string; state: RowState; text: string; settled: boolean };

export type BatchSummary = { headline: string; settled: boolean; rows: BatchRow[]; retry: number };

const SKIPPED_BACKEND = ["DirectModeUnavailable", "UnknownDevice", "AmbiguousDevice", "DeviceMissing", "ZoneOutOfRange"];

/** A target nothing was sent to because of what it is now is skipped; a refused or lost request failed. */
function skipped(error: OutputError): boolean {
  if ("NoEffect" in error || "TooManyTargets" in error) {
    return true;
  }
  if (!("Session" in error)) {
    return false;
  }
  const session = error.Session;
  if (typeof session !== "object") {
    return false;
  }
  if ("OutputConflict" in session || "IdentifyInProgress" in session) {
    return true;
  }
  if (!("Backend" in session)) {
    return false;
  }
  const backend: unknown = session.Backend;
  return typeof backend === "object" && backend !== null && SKIPPED_BACKEND.some((kind) => kind in backend);
}

export function labelOf(session: SessionSnapshot, names: Names, target: OutputTarget): string {
  const device = deviceOf(session, target);
  if (!device) {
    return target.device;
  }
  const where = distinction(session.devices, (each) => names.device(each))[session.devices.indexOf(device)];
  const name = where ? `${names.device(device)} (${where})` : names.device(device);
  return target.zone === null ? name : `${targetName(names, device, target.zone)} on ${name}`;
}

function row(session: SessionSnapshot, names: Names, result: TargetResult, server: string): BatchRow {
  const label = labelOf(session, names, result.target);
  const key = `${result.target.device}|${result.target.zone ?? "all"}`;
  const outcome = result.outcome;
  if (outcome === "Applied") {
    return { key, label, state: "Applied", text: `${label}: accepted.`, settled: true };
  }
  if ("Stopped" in outcome) {
    const text = stopText(outcome.Stopped.restore, server);
    return { key, label, state: "Stopped", text: `${label}: ${text.headline}${text.details.length ? ` ${text.details.join(" ")}` : ""}`, settled: text.settled };
  }
  if ("Covered" in outcome) {
    const by = outcome.Covered.by;
    const why = by.zone === null && result.target.zone !== null ? "the whole device is also selected" : "it is listed twice";
    return { key, label, state: "Covered", text: `${label}: sent once, because ${why}.`, settled: true };
  }
  const state: RowState = skipped(outcome.Skipped.error) ? "Skipped" : "Failed";
  return { key, label, state, text: `${label}: ${state.toLowerCase()}. ${failureText(outcome.Skipped.error)}`, settled: false };
}

export function summarize(batch: BatchFeedback, session: SessionSnapshot, names: Names, server: string): BatchSummary | null {
  if (batch.kind === "failed") {
    return null;
  }
  const rows = batch.report.results.map((result) => row(session, names, result, server));
  const sent = rows.filter((each) => each.state !== "Covered");
  const done = sent.filter((each) => each.state === "Applied" || each.state === "Stopped").length;
  const retry = batch.report.results.filter(
    (result) => typeof result.outcome === "object" && "Skipped" in result.outcome && retryable(result.outcome.Skipped.error),
  ).length;
  const settled = rows.every((each) => each.settled);
  if (batch.kind === "applied") {
    const what = `${batch.label}: ${done} of ${sent.length} ${sent.length === 1 ? "target" : "targets"} accepted.`;
    return { headline: what, settled, rows, retry };
  }
  if (batch.all) {
    const headline = sent.length === 0 ? "No PrismForge effect was running." : `Stop all: ${done} of ${sent.length} ${sent.length === 1 ? "device" : "devices"} stopped.`;
    return { headline, settled, rows, retry };
  }
  return { headline: `${done} of ${sent.length} ${sent.length === 1 ? "target" : "targets"} stopped.`, settled, rows, retry };
}

/** Effects other than the chosen one that Apply would replace, counted by target. */
export function replaced(session: SessionSnapshot, targets: OutputTarget[], chosen: string | null): number {
  return targets.filter((target) => {
    const running = runningOn(session, target);
    return running !== null && running.effect.effect !== chosen;
  }).length;
}

/** Names the targets, a whole device as "(whole device)" and zones of one device as "3 zones of …". */
export function namesLine(session: SessionSnapshot, names: Names, targets: OutputTarget[]): string {
  const devices = [...new Set(targets.map((target) => target.device))];
  const labels = devices.map((device) => {
    const zones = targets.filter((target) => target.device === device && target.zone !== null);
    const whole = labelOf(session, names, { device, zone: null });
    if (zones.length === 0) {
      return whole.endsWith(")") ? `${whole.slice(0, -1)}, whole device)` : `${whole} (whole device)`;
    }
    return zones.length === 1 ? labelOf(session, names, zones[0]!) : `${zones.length} zones of ${whole}`;
  });
  if (labels.length <= 3) {
    return labels.join(", ");
  }
  return `${labels.slice(0, 3).join(", ")} and ${labels.length - 3} more`;
}
