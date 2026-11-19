import { api, type Answer, type Failure } from "../api/client";
import type { DeviceRef } from "../api/bindings/DeviceRef";
import type { IdentifyOutcome } from "../api/bindings/IdentifyOutcome";
import type { ProfileId } from "../api/bindings/ProfileId";
import type { ShellView } from "../api/bindings/ShellView";
import { GroupsState } from "../../features/effects/groups.svelte";
import { EffectsState } from "../../features/effects/state.svelte";
import { ProfilesState } from "../../features/profiles/state.svelte";
import { LifecycleState } from "../../features/settings/state.svelte";
import { AcrossState } from "../../features/setup/across.svelte";
import { SetupState } from "../../features/setup/state.svelte";
import { isLive } from "../../features/devices/present";

export type Work = "connect" | "demo" | "disconnect" | "refresh";

export type Screen = "devices" | "effects" | "setup" | "profiles" | "settings";

export type IdentifyTarget = { device: DeviceRef; zone: number | null };

/** A rename or reset that a context menu asked the device page to start in its name editor. */
export type NameRequest = { device: DeviceRef; zone: number | null; action: "rename" | "clear" };

export type IdentifyState =
  | { kind: "running"; target: IdentifyTarget }
  | { kind: "finished"; target: IdentifyTarget; outcome: IdentifyOutcome }
  | { kind: "failed"; target: IdentifyTarget; error: Failure };

export class AppState {
  view = $state<ShellView>({ revision: 0, session: null, applying: null, connect_apply: null, on_connect: "NotApplicable" });
  working = $state<Work | null>(null);
  problem = $state<Failure | null>(null);
  identify = $state<IdentifyState | null>(null);
  identifiedOnce = $state(false);
  selected = $state<string | null>(null);
  naming = $state<NameRequest | null>(null);
  diagnosticsOpen = $state(false);
  /** True when the window is too narrow to dock Diagnostics, so an open drawer covers the page. */
  layoutCovers = $state(false);
  screen = $state<Screen>("devices");
  connectApplyDismissed = $state(false);
  readonly profiles = new ProfilesState();
  readonly lifecycle = new LifecycleState();
  readonly groups = new GroupsState();
  readonly setup = new SetupState(() => this.view.session);
  readonly across = new AcrossState(
    () => this.view,
    this.setup,
    () => this.effects.catalog,
    () => this.profiles.refreshSoon(),
  );
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

  get covering() {
    return this.diagnosticsOpen && this.layoutCovers;
  }

  uncover() {
    if (this.covering) {
      this.diagnosticsOpen = false;
    }
  }

  async attach(): Promise<() => void> {
    const [unlisten, stopLifecycle] = await Promise.all([
      api.onView((view) => this.accept(view)),
      api.onLifecycle((view) => this.lifecycle.accept(view)),
    ]);
    void this.load();
    return () => {
      unlisten();
      stopLifecycle();
    };
  }

  private async load() {
    const current = await api.view();
    if (current.ok) {
      this.accept(current.value);
    }
    await this.lifecycle.load();
  }

  accept(view: ShellView) {
    if (view.revision <= this.view.revision) {
      return;
    }
    this.view = view;
    if (view.session !== null) {
      void this.effects.load();
    }
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

  async go(screen: Screen): Promise<void> {
    if (screen === "profiles") {
      return this.openProfiles();
    }
    this.uncover();
    this.screen = screen;
  }

  async setLighting(device: DeviceRef) {
    await this.effects.load();
    this.effects.select({ device, zone: null });
    await this.go("effects");
  }

  dismissProblem() {
    this.problem = null;
  }

  openProfiles() {
    this.uncover();
    this.screen = "profiles";
    return this.profiles.refresh();
  }

  async saveActiveChanges(id: ProfileId) {
    await this.openProfiles();
    await this.profiles.select(id);
    await this.profiles.previewUpdate(id);
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
