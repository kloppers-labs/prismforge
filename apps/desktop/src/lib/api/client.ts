import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CommandError } from "./bindings/CommandError";
import type { DeviceRef } from "./bindings/DeviceRef";
import type { IdentifyOutcome } from "./bindings/IdentifyOutcome";
import type { ShellView } from "./bindings/ShellView";

export const VIEW_EVENT = "session-view";
export const DEFAULT_HOST = "127.0.0.1";
export const DEFAULT_PORT = 6742;

export type Failure = CommandError | { Unexpected: string };
export type Answer<T> = { ok: true; value: T } | { ok: false; error: Failure };

async function call<T>(command: string, args?: Record<string, unknown>): Promise<Answer<T>> {
  try {
    return { ok: true, value: await invoke<T>(command, args) };
  } catch (error) {
    return { ok: false, error: asFailure(error) };
  }
}

function asFailure(error: unknown): Failure {
  if (error === "NoSession" || error === "EmptyHost") {
    return error;
  }
  if (
    typeof error === "object" &&
    error !== null &&
    ("Session" in error || "InvalidEffect" in error || "NoEffect" in error)
  ) {
    return error as CommandError;
  }
  return { Unexpected: String(error) };
}

export const api = {
  view: () => call<ShellView>("view"),
  connectOpenRgb: (host: string, port: number) => call<ShellView>("connect_openrgb", { host, port }),
  startDemo: () => call<ShellView>("start_demo"),
  disconnect: () => call<ShellView>("disconnect"),
  refresh: () => call<null>("refresh"),
  identify: (device: DeviceRef, zone: number | null) =>
    call<IdentifyOutcome>("identify", { device, zone }),
  onView: (handler: (view: ShellView) => void): Promise<UnlistenFn> =>
    listen<ShellView>(VIEW_EVENT, (event) => handler(event.payload)),
};
