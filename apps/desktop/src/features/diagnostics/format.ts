import type { BackendError } from "../../lib/api/bindings/BackendError";
import type { SessionEventKind } from "../../lib/api/bindings/SessionEventKind";

export function raw(value: unknown): string {
  return typeof value === "string" ? value : JSON.stringify(value);
}

export function errorName(error: BackendError): string {
  return typeof error === "string" ? error : (Object.keys(error)[0] ?? "Unknown");
}

export function eventText(kind: SessionEventKind): string {
  if (typeof kind === "string") {
    return kind;
  }
  if ("ConnectFailed" in kind) {
    return `ConnectFailed ${raw(kind.ConnectFailed.error)}`;
  }
  if ("ConnectionLost" in kind) {
    return `ConnectionLost ${raw(kind.ConnectionLost.reason)}`;
  }
  if ("RetryScheduled" in kind) {
    return `RetryScheduled attempt ${kind.RetryScheduled.attempt} in ${kind.RetryScheduled.delay_ms} ms`;
  }
  if ("InventoryRefreshed" in kind) {
    const { generation, devices } = kind.InventoryRefreshed;
    return `InventoryRefreshed generation ${generation}, ${devices} devices`;
  }
  if ("RefreshFailed" in kind) {
    return `RefreshFailed ${raw(kind.RefreshFailed.error)}`;
  }
  if ("IdentifyStarted" in kind) {
    const { device, zone } = kind.IdentifyStarted;
    return `IdentifyStarted ${device}${zone === null ? "" : ` zone ${zone}`}`;
  }
  return `IdentifyFinished ${raw(kind.IdentifyFinished.outcome.restore)}`;
}
