import type { IdentifyOutcome } from "../../lib/api/bindings/IdentifyOutcome";
import type { RestoreFailure } from "../../lib/api/bindings/RestoreFailure";
import { backendErrorText } from "../../lib/errors";

export type OutcomeText = {
  settled: boolean;
  headline: string;
  details: string[];
};

export function describeOutcome(outcome: IdentifyOutcome): OutcomeText {
  const restore = outcome.restore;
  const lightFailure = outcome.light_failure ? backendErrorText(outcome.light_failure) : null;
  if (restore === "Unchanged") {
    return {
      settled: true,
      headline: "Nothing was changed on the device.",
      details: lightFailure ? [lightFailure] : [],
    };
  }
  const lit = lightFailure
    ? [`Lighting the device failed partway: ${lightFailure}`]
    : [];
  if (restore === "Restored") {
    return {
      settled: true,
      headline: "Identify finished. OpenRGB accepted the requests to put back the earlier lighting.",
      details: [
        ...lit,
        "PrismForge cannot confirm what the device shows now. If other software changed the lighting since the last device scan, that older state came back.",
      ],
    };
  }
  if ("PartlyRestored" in restore) {
    const failure = restore.PartlyRestored.failure;
    if (failure === "PreviousStateUnknown") {
      return {
        settled: false,
        headline: "Identify finished, but the earlier lighting may not be back.",
        details: [
          ...lit,
          "PrismForge did not know the earlier lighting of this device. OpenRGB reported no active mode, or an earlier identify was not undone. PrismForge put back what it had.",
          "Check the device, and set its lighting again in OpenRGB if needed.",
        ],
      };
    }
    return {
      settled: false,
      headline: "Only part of the earlier lighting was put back.",
      details: [...lit, failureText(failure), "Set the lighting again in OpenRGB if the device looks wrong."],
    };
  }
  return {
    settled: false,
    headline: "The earlier lighting was not put back. The device may stay white.",
    details: [...lit, failureText(restore.NotRestored.failure), "Set the lighting again in OpenRGB, or run identify again."],
  };
}

function failureText(failure: RestoreFailure): string {
  if (failure === "PreviousStateUnknown") {
    return "PrismForge did not know the earlier lighting of this device.";
  }
  if (failure === "TargetChanged") {
    return "The device changed during a rescan, so PrismForge wrote nothing back to it.";
  }
  if ("ConnectionLost" in failure) {
    return "The connection to OpenRGB was lost before PrismForge could put the lighting back.";
  }
  return `OpenRGB refused the request to put the lighting back. ${backendErrorText(failure.Refused.error)}`;
}
