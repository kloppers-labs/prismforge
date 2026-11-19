import { emit } from "@tauri-apps/api/event";
import { cleanup } from "@testing-library/svelte";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach } from "vitest";
import { VIEW_EVENT } from "../lib/api/client";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../lib/api/bindings/ShellView";
import { storeEmpty } from "./fixtures/stores";

export type Call = { command: string; args: Record<string, unknown> };
type Handler = (args: Record<string, unknown>) => unknown;

afterEach(() => {
  cleanup();
  clearMocks();
});

/** Stands in for the Tauri transport only; every component above it is real. */
export function fakeShell(initial: SessionSnapshot | null, handlers: Record<string, Handler> = {}) {
  const calls: Call[] = [];
  let revision = 1;
  const view = (session: SessionSnapshot | null): ShellView => ({
    revision: ++revision,
    session,
    applying: null,
    connect_apply: null,
  });
  mockIPC(
    (command, args) => {
      const plain = (args ?? {}) as Record<string, unknown>;
      calls.push({ command, args: plain });
      if (command === "view") {
        return { revision: 1, session: initial, applying: null, connect_apply: null };
      }
      const handler = handlers[command] ?? (command === "profile_store" ? () => storeEmpty : undefined);
      if (!handler) {
        throw new Error(`unexpected command ${command}`);
      }
      return handler(plain);
    },
    { shouldMockEvents: true },
  );
  return {
    calls,
    view,
    commands: () => calls.map((call) => call.command),
    push: (session: SessionSnapshot | null) => emit(VIEW_EVENT, view(session)),
    pushView: (shellView: ShellView) => emit(VIEW_EVENT, shellView),
  };
}

export function refuse(error: unknown): Handler {
  return () => Promise.reject(error);
}
