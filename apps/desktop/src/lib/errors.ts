import type { Failure } from "./api/client";
import type { BackendError } from "./api/bindings/BackendError";

export function failureText(failure: Failure): string {
  if (failure === "NoSession") {
    return "PrismForge is not connected. Connect first.";
  }
  if (failure === "EmptyHost") {
    return "Enter the address of the computer that runs OpenRGB.";
  }
  if ("Unexpected" in failure) {
    return `PrismForge could not run this request: ${failure.Unexpected}`;
  }
  const error = failure.Session;
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
  return backendErrorText(error.Backend);
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
