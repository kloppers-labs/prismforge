import type { AppState } from "../state/app.svelte";
import { failureText } from "../errors";
import { summarise, type Tone } from "../../features/connection/wording";
import { connectResultText, fullyApplied } from "../../features/profiles/present";
import { lifecycleFailureText, trayStopTitle } from "../../features/settings/wording";
import { ended } from "./status";
import { failedRunTitle } from "../../features/automation/wording";

export type NoticeKind = "problem" | "lifecycle" | "connection" | "repaired" | "trayStop" | "connectApply" | "automation";

export type NoticeItem = { kind: NoticeKind; tone: Tone; title: string };

const RANK: Record<Tone, number> = { problem: 0, warning: 1, waiting: 2, working: 3, ok: 4 };

export const REPAIRED_TITLE = "PrismForge repaired stored files. Open Profiles to read what happened and what to check.";

/** Every notice the shell holds now; a notice leaves only when its data does or the user dismisses it. */
export function activeNotices(app: AppState): NoticeItem[] {
  const items: NoticeItem[] = [];
  const session = app.session;
  const failure = app.lifecycle.failure;
  const store = app.profiles.store;
  const stop = app.lifecycle.trayStop;
  const applied = app.connectApplyDismissed ? null : app.view.connect_apply;
  if (app.problem !== null) {
    items.push({ kind: "problem", tone: "problem", title: failureText(app.problem) });
  }
  if (failure !== null && (failure.key === "close" || failure.key === "quit")) {
    items.push({ kind: "lifecycle", tone: "problem", title: lifecycleFailureText(failure.key, failureText(failure.error)) });
  }
  if (session !== null && !ended(session) && session.devices.length > 0) {
    const summary = summarise(session);
    if (summary.reasons.length > 0) {
      items.push({ kind: "connection", tone: summary.tone, title: summary.reasons[0]! });
    }
  }
  if (store !== null && store.summary.notices.length > 0 && app.screen !== "profiles") {
    items.push({ kind: "repaired", tone: "warning", title: REPAIRED_TITLE });
  }
  if (stop !== null) {
    items.push({ kind: "trayStop", tone: stop.stops.some((each) => "Failed" in each.result) ? "warning" : "ok", title: trayStopTitle(stop) });
  }
  if (applied !== null) {
    const outcome = applied.outcome;
    const tone: Tone = "Applied" in outcome ? (fullyApplied(outcome.Applied.report) ? "ok" : "warning") : "problem";
    items.push({ kind: "connectApply", tone, title: connectResultText(applied) });
  }
  const failed = app.screen === "automation" ? [] : app.automation.unseenWith(app.view.rule_run);
  if (failed.length > 0) {
    items.push({ kind: "automation", tone: "warning", title: failed.map(failedRunTitle).join(" ") });
  }
  return items;
}

/** Orders notices by tone, then newest first, and reports which ones arrived since the last call. */
export class NoticeOrder {
  private seen = new Map<NoticeKind, number>();
  private next = 0;

  order(items: NoticeItem[]): { sorted: NoticeItem[]; arrived: NoticeItem[] } {
    const present = new Set(items.map((item) => item.kind));
    for (const kind of [...this.seen.keys()]) {
      if (!present.has(kind)) {
        this.seen.delete(kind);
      }
    }
    const arrived = items.filter((item) => !this.seen.has(item.kind));
    for (const item of arrived) {
      this.next += 1;
      this.seen.set(item.kind, this.next);
    }
    const sorted = [...items].sort((a, b) => RANK[a.tone] - RANK[b.tone] || this.seen.get(b.kind)! - this.seen.get(a.kind)!);
    return { sorted, arrived };
  }
}
