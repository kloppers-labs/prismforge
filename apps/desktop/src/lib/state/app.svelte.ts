import { api, type Answer, type Failure } from "../api/client";
import type { DeviceRef } from "../api/bindings/DeviceRef";
import type { IdentifyOutcome } from "../api/bindings/IdentifyOutcome";
import type { ShellView } from "../api/bindings/ShellView";
import { EffectsState } from "../../features/effects/state.svelte";
import { ProfilesState } from "../../features/profiles/state.svelte";
import { isLive } from "../../features/devices/present";

export type Work = "connect" | "demo" | "disconnect" | "refresh";

export type Screen = "devices" | "effects" | "profiles";

export type IdentifyTarget = { device: DeviceRef; zone: number | null };

export type IdentifyState =
  | { kind: "running"; target: IdentifyTarget }
  | { kind: "finished"; target: IdentifyTarget; outcome: IdentifyOutcome }
  | { kind: "failed"; target: IdentifyTarget; error: Failure };

export class AppState {
  view = $state<ShellView>({ revision: 0, session: null, applying: null, connect_apply: null });
  working = $state<Work | null>(null);
  problem = $state<Failure | null>(null);
  identify = $state<IdentifyState | null>(null);
  identifiedOnce = $state(false);
  selected = $state<string | null>(null);
  diagnosticsOpen = $state(false);
  screen = $state<Screen>("devices");
  connectApplyDismissed = $state(false);
  readonly profiles = new ProfilesState();
  readonly effects = new EffectsState(
    () => this.view,
    () => this.profiles.names,
    () => this.profiles.refreshSoon(),
  );
  private storeKey = storeKey(this.view);
  private sessionKey = sessionKey(this.view);

  get names() {
    return this.profiles.names;
  }

  get session() {
    return this.view.session;
  }

  async attach(): Promise<() => void> {
    const unlisten = await api.onView((view) => this.accept(view));
    const current = await api.view();
    if (current.ok) {
      this.accept(current.value);
    }
    return unlisten;
  }

  accept(view: ShellView) {
    if (view.revision <= this.view.revision) {
      return;
    }
    this.view = view;
    if (view.connect_apply === null) {
      this.connectApplyDismissed = false;
    }
    const session = sessionKey(view);
    if (session !== this.sessionKey) {
      this.sessionKey = session;
      this.profiles.endScope();
    }
    const key = storeKey(view);
    if (key !== this.storeKey) {
      this.storeKey = key;
      this.storeChanged();
    }
  }

  openProfiles() {
    this.screen = "profiles";
    return this.profiles.refresh();
  }

  /** The store opens on first use, so without a session it is read only when the user opens Profiles. */
  private storeChanged() {
    if (this.view.session === null && this.screen !== "profiles") {
      this.profiles.forget();
    } else {
      this.profiles.refreshSoon();
    }
  }

  connect(host: string, port: number) {
    return this.startSession("connect", () => api.connectOpenRgb(host, port));
  }

  startDemo() {
    return this.startSession("demo", api.startDemo);
  }

  disconnect() {
    this.effects.leave();
    this.profiles.endScope();
    return this.run("disconnect", api.disconnect);
  }

  refresh() {
    return this.run("refresh", api.refresh);
  }

  async identifyTarget(target: IdentifyTarget) {
    this.identify = { kind: "running", target };
    const answer = await api.identify(target.device, target.zone);
    this.identifiedOnce = true;
    this.identify = answer.ok
      ? { kind: "finished", target, outcome: answer.value }
      : { kind: "failed", target, error: answer.error };
  }

  private startSession(work: Work, action: () => Promise<Answer<ShellView>>) {
    this.identify = null;
    this.selected = null;
    this.effects.reset();
    this.profiles.endScope();
    return this.run(work, action);
  }

  private async run<T>(work: Work, action: () => Promise<Answer<T>>) {
    this.working = work;
    this.problem = null;
    const answer = await action();
    this.working = null;
    if (!answer.ok) {
      this.problem = answer.error;
    } else if (isView(answer.value)) {
      this.accept(answer.value);
    }
  }
}

function sessionKey(view: ShellView): string {
  return view.session?.backend.origin ?? "none";
}

function storeKey(view: ShellView): string {
  const session = view.session;
  return [
    session?.backend.origin ?? "none",
    session && isLive(session) ? "live" : "idle",
    session?.devices.length ?? 0,
    view.applying?.id ?? "",
    view.connect_apply ? "applied" : "",
  ].join("|");
}

function isView(value: unknown): value is ShellView {
  return typeof value === "object" && value !== null && "revision" in value;
}
