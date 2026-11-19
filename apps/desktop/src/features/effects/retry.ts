import type { OutputError } from "../../lib/api/bindings/OutputError";

const NEVER_LATER = ["DirectModeUnavailable", "UnknownDevice", "AmbiguousDevice", "ZoneOutOfRange"];

/** A target a second try can reach: not one PrismForge must refuse again until something about it changes. */
export function retryable(error: OutputError): boolean {
  if (!("Session" in error)) {
    return false;
  }
  const session = error.Session;
  if (typeof session !== "object") {
    return true;
  }
  if ("OutputConflict" in session) {
    return false;
  }
  if (!("Backend" in session)) {
    return true;
  }
  const backend: unknown = session.Backend;
  return !(typeof backend === "object" && backend !== null && NEVER_LATER.some((kind) => kind in backend));
}
