import type { GroupsView } from "../lib/api/bindings/GroupsView";
import type { LifecycleView } from "../lib/api/bindings/LifecycleView";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../lib/api/bindings/ShellView";
import type { StoreView } from "../lib/api/bindings/StoreView";
import { isLive } from "../features/devices/present";
import { lifecycleChosen, lifecycleFresh, lifecycleStartPath, lifecycleTrayStopped } from "../test/fixtures/lifecycles";
import * as sessions from "../test/fixtures/sessions";
import { storeEmpty, storeOffline, storeRecovered, storeSaved, storeTwo, storeUnsaved } from "../test/fixtures/stores";
import { viewApplying, viewArmedRetrying, viewConnectApplied } from "../test/fixtures/views";
import type { Handler } from "./transport";

export type Area = "Devices" | "Effects" | "Profiles" | "Settings";

export type Scenario = {
  label: string;
  view: ShellView;
  store: StoreView;
  lifecycle: LifecycleView;
  area?: Area;
  diagnostics?: boolean;
  handlers?: Record<string, Handler>;
  groups?: GroupsView;
  rescan?: boolean;
};

function onConnect(session: SessionSnapshot | null): ShellView["on_connect"] {
  if (session === null) {
    return "NotApplicable";
  }
  return isLive(session) ? { Fired: { ran: null, next: null } } : "NotArmed";
}

function view(session: SessionSnapshot | null, extra: Partial<ShellView> = {}): ShellView {
  return { revision: 1, session, applying: null, connect_apply: null, on_connect: onConnect(session), ...extra };
}

const base = { store: storeEmpty, lifecycle: lifecycleFresh };

/** OpenRGB sessions read the real store; the generated store fixtures are demo stores, so the preview relabels a copy. */
function real(store: StoreView): StoreView {
  return { ...structuredClone(store), kind: "Real" };
}

const openRgb = { store: real(storeEmpty), lifecycle: lifecycleFresh };

function longNames(): { session: SessionSnapshot; store: StoreView } {
  const session = structuredClone(sessions.openRgbConnected);
  session.backend.address = "lighting-controller.living-room.home.arpa:16742";
  const store = structuredClone(storeUnsaved);
  store.kind = "Real";
  const active = store.summary.profiles.find((profile) => profile.id === store.summary.active);
  if (active) {
    active.name = "Evening with the desk lamps and the long hallway strip";
  }
  return { session, store };
}

const long = longNames();

export const SCENARIOS: Record<string, Scenario> = {
  "first-run": { ...base, label: "First run, not connected", view: view(null), store: storeOffline },
  "first-run-remembered": { ...base, label: "Not connected, last address remembered", view: view(null), store: storeOffline, lifecycle: lifecycleStartPath },
  "effects-demo-diagnostics": { ...base, label: "Effects with Diagnostics open", view: view(sessions.demoEffectsRunning), store: storeSaved, area: "Effects", diagnostics: true },
  connecting: { ...openRgb, label: "Connecting to OpenRGB", view: view(sessions.openRgbConnecting) },
  retrying: { ...openRgb, label: "Retrying, nothing answers", view: view(sessions.openRgbRefused) },
  lost: { ...openRgb, label: "Connection lost, devices kept", view: view(sessions.openRgbLostWhileConnected) },
  "old-server": { ...openRgb, label: "Stopped: old OpenRGB server", view: view(sessions.openRgbOldServer) },
  connected: { ...base, label: "Connected to OpenRGB", view: view(sessions.openRgbConnected), store: storeOffline },
  degraded: { ...openRgb, label: "Connected with problems", view: view(sessions.openRgbDegraded) },
  demo: { ...base, label: "Demo inventory", view: view(sessions.demoConnected) },
  "demo-unsaved": { ...base, label: "Demo, active profile with unsaved changes", view: view(sessions.demoConnected), store: storeUnsaved },
  "effects-running": { ...base, label: "Effects running", view: view(sessions.demoEffectsRunning), store: storeSaved, area: "Effects" },
  "effects-offline": { ...base, label: "Effects without a connection", view: view(null), area: "Effects" },
  "profiles-unresolved": { ...base, label: "Profiles with targets not found", view: view(sessions.demoConnected), store: storeTwo, area: "Profiles" },
  applying: { ...base, label: "Profile apply in progress", view: viewApplying, store: storeSaved },
  "armed-retrying": { ...openRgb, label: "Apply on connect armed, retrying", view: viewArmedRetrying, store: storeOffline },
  notices: {
    ...base,
    label: "Several notices",
    view: viewConnectApplied,
    store: storeRecovered,
    lifecycle: lifecycleTrayStopped,
    handlers: { refresh: () => Promise.reject({ Session: "Busy" }) },
    rescan: true,
  },
  settings: { ...openRgb, label: "Settings", view: view(sessions.openRgbConnected), lifecycle: lifecycleStartPath, area: "Settings" },
  "settings-background": { ...base, label: "Settings, keep running chosen", view: view(sessions.demoConnected), lifecycle: lifecycleChosen, area: "Settings" },
  diagnostics: { ...openRgb, label: "Diagnostics drawer", view: view(sessions.openRgbDegraded), diagnostics: true },
  "long-names": { ...base, label: "Long server address and profile name", view: view(long.session), store: long.store },
};

export const DEFAULT_SCENARIO = "demo";
