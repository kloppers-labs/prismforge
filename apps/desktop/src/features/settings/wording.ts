import type { OpenRgbAddress } from "../../lib/api/bindings/OpenRgbAddress";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
import type { StopReport } from "../../lib/api/bindings/StopReport";
import type { StopResult } from "../../lib/api/bindings/StopResult";
import { failureText, fileProblemText } from "../../lib/errors";
import { stopText } from "../effects/present";

export function isMac(): boolean {
  return /Mac/i.test(navigator.userAgent);
}

export function quitShortcut(): string {
  return isMac() ? "Command-Q" : "Ctrl+Q";
}

export function trayName(): string {
  return isMac() ? "the menu bar" : "the system tray";
}

export function addressText(address: OpenRgbAddress): string {
  return `${address.host}:${address.port}`;
}

export function settingsNoticeText(notice: StoreNotice): string[] {
  const event = notice.event;
  if ("RestoredFromBackup" in event) {
    const { problem, set_aside } = event.RestoredFromBackup;
    return [
      problem === null
        ? `${notice.file} was missing, so PrismForge used its backup.`
        : `PrismForge could not read ${notice.file} and used its backup. ${fileProblemText(problem)}`,
      "Your last settings change may be missing. Check the settings on this screen.",
      ...(set_aside ? [`The damaged file is still there, now called ${set_aside}.`] : []),
    ];
  }
  if ("StartedEmpty" in event) {
    const { set_aside } = event.StartedEmpty;
    return [
      `PrismForge could not read ${notice.file} or its backup, so every setting on this screen is off.`,
      ...(set_aside.length > 0 ? [`The damaged files are still there: ${set_aside.join(", ")}.`] : []),
    ];
  }
  if ("LeftUntouched" in event) {
    return [
      `${notice.file} needs a newer PrismForge. This version does not change it, so every setting on this screen is off.`,
    ];
  }
  const failure = event.WriteFailed.failure;
  return [`PrismForge could not save ${failure.file}: ${failure.detail}.`, "The previous version is still in place."];
}

export function trayStopTitle(report: StopReport): string {
  if (report.stops.length === 0) {
    return "Stop all PrismForge effects, from the tray: no PrismForge effect was running.";
  }
  return report.stops.some((stop) => "Failed" in stop.result)
    ? "Stop all PrismForge effects, from the tray: some effects were not stopped."
    : "Stop all PrismForge effects, from the tray: every effect was stopped.";
}

export function stopResultText(result: StopResult, server: string): string[] {
  if ("Failed" in result) {
    return [`Not stopped. ${failureText(result.Failed.error)}`];
  }
  const text = stopText(result.Stopped.restore, server);
  return [text.headline, ...text.details];
}

export type StartPath = {
  system: boolean | null;
  hidden: boolean;
  connect: boolean;
  address: string | null;
  profile: string | null;
};

export function startPathText(path: StartPath): string {
  if (path.system === null) {
    return "PrismForge cannot tell whether your lights change at system start, because it cannot read its start entry.";
  }
  if (path.system && path.connect && path.address !== null && path.profile !== null) {
    const start = path.hidden ? `starts hidden in ${trayName()}` : "starts";
    return `When you sign in, PrismForge ${start}, connects to OpenRGB at ${path.address} and applies ${path.profile}. Your lights change at system start without a click.`;
  }
  const missing = [
    ...(path.system ? [] : ["start PrismForge when you sign in"]),
    ...(path.connect ? [] : ["connect to OpenRGB when PrismForge starts"]),
    ...(path.profile !== null ? [] : ["a profile marked to apply when PrismForge connects"]),
  ];
  return `Your lights do not change by themselves at system start. Not set: ${missing.join(", ")}.`;
}

export const MACHINE_ENTRY_REMOVAL = 'reg delete "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run" /v PrismForge /f';

export function lifecycleFailureText(key: "close" | "quit", failure: string): string {
  return key === "close"
    ? `PrismForge could not remember your choice for closing the window. ${failure} It asks again the next time you close the window.`
    : `PrismForge could not start to quit. ${failure}`;
}
