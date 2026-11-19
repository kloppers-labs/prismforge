import type { OpenRgbAddress } from "../../lib/api/bindings/OpenRgbAddress";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
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

export function stopResultText(result: StopResult, server: string): string[] {
  if ("Failed" in result) {
    return [`Not stopped. ${failureText(result.Failed.error)}`];
  }
  const text = stopText(result.Stopped.restore, server);
  return [text.headline, ...text.details];
}
