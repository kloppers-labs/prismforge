import type { Failure } from "./api/client";
import type { BackendError } from "./api/bindings/BackendError";
import type { FileProblem } from "./api/bindings/FileProblem";
import type { FileRule } from "./api/bindings/FileRule";
import type { NameProblem } from "./api/bindings/NameProblem";
import type { PlanError } from "./api/bindings/PlanError";
import type { SessionError } from "./api/bindings/SessionError";
import type { SettingProblem } from "./api/bindings/SettingProblem";
import type { StoreError } from "./api/bindings/StoreError";
import type { StoreUnavailable } from "./api/bindings/StoreUnavailable";

export function failureText(failure: Failure): string {
  if (failure === "NoSession") {
    return "PrismForge is not connected. Connect first.";
  }
  if (failure === "EmptyHost") {
    return "Enter the address of the computer that runs OpenRGB.";
  }
  if (failure === "ProfileApplying") {
    return "PrismForge is applying a profile. Wait until it finishes, then try again.";
  }
  if (failure === "NotConnected") {
    return "PrismForge is not connected, so the profile was not applied.";
  }
  if ("Unexpected" in failure) {
    return `PrismForge could not run this request: ${failure.Unexpected}`;
  }
  if ("InvalidEffect" in failure) {
    return "Some effect settings are not valid. The settings below show which.";
  }
  if ("NoEffect" in failure) {
    return "No PrismForge effect runs on this target.";
  }
  if ("TooManyTargets" in failure) {
    return `PrismForge takes at most ${failure.TooManyTargets.limit} targets at once. Select fewer targets, then try again.`;
  }
  if ("Store" in failure) {
    return storeErrorText(failure.Store);
  }
  if ("StoreUnavailable" in failure) {
    return storeUnavailableText(failure.StoreUnavailable);
  }
  if ("Unreadable" in failure) {
    return `The file cannot be imported. ${fileProblemText(failure.Unreadable.problem)} Nothing was stored.`;
  }
  if ("File" in failure) {
    return fileRuleText(failure.File);
  }
  if ("Setting" in failure) {
    return settingProblemText(failure.Setting);
  }
  if ("Plan" in failure) {
    return planErrorText(failure.Plan);
  }
  if ("PreviewTooLarge" in failure) {
    return `The draft preview draws at most ${failure.PreviewTooLarge.limit} LEDs, so it shows nothing for this setup. Your devices stay as they are.`;
  }
  return sessionErrorText(failure.Session);
}

export function planErrorText(error: PlanError): string {
  if ("UnknownView" in error) {
    return "The saved setup has no such view, so nothing was sent. Save the setup first.";
  }
  if ("InvalidSetup" in error) {
    return "The saved setup is not valid, so nothing was sent.";
  }
  return "The flow is outside its range: an angle from 0 to 359 degrees, or a centre from 0 to 100 percent. Nothing was sent.";
}

export function settingProblemText(problem: SettingProblem): string {
  if (problem === "TrayUnavailable") {
    return "This system has no tray, so PrismForge cannot keep running without its window. Closing the window quits PrismForge.";
  }
  if (problem === "NoAddress") {
    return "Connect to OpenRGB once first. PrismForge then remembers that address and can connect to it when it starts.";
  }
  const change = problem.StartEntry.wanted ? "add PrismForge to the programs that start with the system" : "remove PrismForge from the programs that start with the system";
  if (problem.StartEntry.detail === null) {
    return `PrismForge tried to ${change}, but the system did not keep the change. The switch shows what the system reports now.`;
  }
  return `PrismForge could not ${change}: ${problem.StartEntry.detail}. The switch shows what the system reports now.`;
}

export function sessionErrorText(error: SessionError): string {
  if (error === "Busy") {
    return "PrismForge is still working on earlier requests. Wait a moment, then try again.";
  }
  if (error === "Stopped") {
    return "The connection was closed before this request finished.";
  }
  if ("IdentifyInProgress" in error) {
    return "Another identify is still running. Wait until it finishes.";
  }
  if ("InvalidDuration" in error) {
    return "The identify time is outside the allowed range.";
  }
  if ("OutputConflict" in error) {
    return "You said another program controls this device. PrismForge sends nothing to it until you hand it back.";
  }
  return backendErrorText(error.Backend);
}

export function storeErrorText(error: StoreError): string {
  if ("UnknownProfile" in error) {
    return "This profile is not in the store any more.";
  }
  if ("NewerFile" in error) {
    return `${error.NewerFile.file} needs a newer PrismForge, so this version does not change it.`;
  }
  if ("InvalidName" in error) {
    return nameProblemText(error.InvalidName.problem);
  }
  if ("NameTaken" in error) {
    return `Another profile is already called ${error.NameTaken.name}. Choose another name.`;
  }
  if ("UnknownGroup" in error) {
    return "This group is not in the store any more.";
  }
  if ("GroupNameTaken" in error) {
    return `Another group is already called ${error.GroupNameTaken.name}. Choose another name.`;
  }
  if ("TooManyGroups" in error) {
    return `PrismForge keeps at most ${error.TooManyGroups.limit} groups. Delete a group first.`;
  }
  if ("TooManyMembers" in error) {
    return `A group holds at most ${error.TooManyMembers.limit} targets. Select fewer targets.`;
  }
  if ("UnknownDevice" in error) {
    return "PrismForge has not stored this device yet. Try again after the next device scan.";
  }
  if ("UnknownZone" in error) {
    return "This zone is not on the device.";
  }
  if ("InvalidProfile" in error || "InvalidRegistry" in error || "InvalidSettings" in error || "InvalidGroups" in error || "InvalidSetup" in error) {
    return "PrismForge did not write the file, because the result would not be valid. The previous version is still in place.";
  }
  if ("InvalidRule" in error) {
    return error.InvalidRule.problem === "LightsOffNotForThisEvent"
      ? "Switching the lights off is not an action for this event."
      : "This rule was set by a newer PrismForge, so this version cannot change its action.";
  }
  if ("NotActive" in error) {
    return "This profile was not applied or saved in this session, so PrismForge cannot tell which changes belong to it.";
  }
  if ("SessionUnavailable" in error) {
    return `PrismForge could not read your changes from the session, so nothing was saved. ${sessionErrorText(error.SessionUnavailable.error)}`;
  }
  if ("TooLarge" in error) {
    return `${error.TooLarge.file} would be larger than the limit of ${mebibytes(error.TooLarge.limit)}, so nothing was written.`;
  }
  if ("Write" in error) {
    return `PrismForge could not write ${error.Write.file}: ${error.Write.detail}. The previous version is still in place.`;
  }
  if ("Remove" in error) {
    return `PrismForge could not remove ${error.Remove.file}: ${error.Remove.detail}.`;
  }
  return `PrismForge could not prepare its profile folder: ${error.Folder.detail}.`;
}

export function storeUnavailableText(error: StoreUnavailable): string {
  if (error === "InUse") {
    return "Another PrismForge window is using the profile folder. This window does not read or save profiles. Close the other window, then try again.";
  }
  return `PrismForge could not open its profile folder: ${error.Folder.detail}.`;
}

export function nameProblemText(problem: NameProblem): string {
  if (problem === "Empty") {
    return "Enter a name.";
  }
  if (problem === "ControlCharacter") {
    return "Use letters, numbers, spaces and punctuation only.";
  }
  return `Use at most ${problem.TooLong.max} characters.`;
}

export function fileProblemText(problem: FileProblem): string {
  if (problem === "NoVersion") {
    return "It has no schema version.";
  }
  if ("Unreadable" in problem) {
    return `It could not be read: ${problem.Unreadable.detail}.`;
  }
  if ("TooLarge" in problem) {
    return `It is larger than ${mebibytes(problem.TooLarge.limit)}.`;
  }
  if ("NotJson" in problem) {
    return "It is not a JSON file.";
  }
  if ("WrongFormat" in problem) {
    return "It is not a PrismForge profile export.";
  }
  if ("NewerVersion" in problem) {
    return "A newer PrismForge made it.";
  }
  if ("UnknownOldVersion" in problem) {
    return "Its schema version is older than this PrismForge reads.";
  }
  return `Its content is not valid: ${problem.Invalid.detail}.`;
}

function fileRuleText(rule: FileRule): string {
  if (rule === "NotAbsolute" || rule === "NotAFile") {
    return "PrismForge cannot use this location. Choose a file in a folder of your own.";
  }
  if (rule === "InStoreFolder") {
    return "This location is inside PrismForge's own folder. Choose a place outside it.";
  }
  if ("TooLarge" in rule) {
    return `The file is larger than ${mebibytes(rule.TooLarge.limit)}, so PrismForge did not read it.`;
  }
  return `PrismForge cannot use this location: ${rule.Unreadable.detail}.`;
}

function mebibytes(bytes: number): string {
  return `${bytes / (1024 * 1024)} MiB`;
}

export function backendErrorText(error: BackendError): string {
  if (error === "NotConnected") {
    return "PrismForge is not connected to the server right now.";
  }
  if (error === "StaleInventory") {
    return "The device list was being updated. Try again in a moment; it works after the scan finishes.";
  }
  if ("Unreachable" in error) {
    return "The server could not be reached.";
  }
  if ("UnsupportedProtocol" in error) {
    return "The server does not support the required protocol.";
  }
  if ("Timeout" in error) {
    return "The server did not answer in time.";
  }
  if ("ConnectionLost" in error) {
    return "The connection to the server was lost.";
  }
  if ("ProtocolViolation" in error) {
    return "The server sent data PrismForge could not read.";
  }
  if ("Rejected" in error) {
    return "OpenRGB refused the request.";
  }
  if ("UnknownDevice" in error || "DeviceMissing" in error) {
    return "The device is not in the current device list.";
  }
  if ("AmbiguousDevice" in error) {
    return "PrismForge cannot tell this device apart from an identical one.";
  }
  if ("DirectModeUnavailable" in error) {
    return "This device has no Direct Mode.";
  }
  if ("ZoneOutOfRange" in error) {
    return "This zone is not on the device any more.";
  }
  if ("ModeOutOfRange" in error) {
    return "This mode is not on the device any more.";
  }
  return "The number of colours did not match the number of LEDs.";
}
