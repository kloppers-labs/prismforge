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

export const SCENARIOS: Record<string, Scenario> = {
  "first-run": { ...base, label: "First run, not connected", view: view(null), store: storeOffline },
  connecting: { ...base, label: "Connecting to OpenRGB", view: view(sessions.openRgbConnecting) },
  retrying: { ...base, label: "Retrying, nothing answers", view: view(sessions.openRgbRefused) },
  lost: { ...base, label: "Connection lost, devices kept", view: view(sessions.openRgbLostWhileConnected) },
  "old-server": { ...base, label: "Stopped: old OpenRGB server", view: view(sessions.openRgbOldServer) },
  connected: { ...base, label: "Connected to OpenRGB", view: view(sessions.openRgbConnected), store: storeOffline },
  degraded: { ...base, label: "Connected with problems", view: view(sessions.openRgbDegraded) },
  demo: { ...base, label: "Demo inventory", view: view(sessions.demoConnected) },
  "demo-unsaved": { ...base, label: "Demo, active profile with unsaved changes", view: view(sessions.demoConnected), store: storeUnsaved },
  "effects-running": { ...base, label: "Effects running", view: view(sessions.demoEffectsRunning), store: storeSaved, area: "Effects" },
  "effects-offline": { ...base, label: "Effects without a connection", view: view(null), area: "Effects" },
  "profiles-unresolved": { ...base, label: "Profiles with targets not found", view: view(sessions.demoConnected), store: storeTwo, area: "Profiles" },
  applying: { ...base, label: "Profile apply in progress", view: viewApplying, store: storeSaved },
  "armed-retrying": { ...base, label: "Apply on connect armed, retrying", view: viewArmedRetrying, store: storeOffline },
  notices: {
    ...base,
    label: "Several notices",
    view: viewConnectApplied,
    store: storeRecovered,
    lifecycle: lifecycleTrayStopped,
    handlers: { refresh: () => Promise.reject({ Session: "Busy" }) },
    rescan: true,
  },
  settings: { ...base, label: "Settings", view: view(sessions.openRgbConnected), lifecycle: lifecycleStartPath, area: "Settings" },
  "settings-background": { ...base, label: "Settings, keep running chosen", view: view(sessions.demoConnected), lifecycle: lifecycleChosen, area: "Settings" },
  diagnostics: { ...base, label: "Diagnostics drawer", view: view(sessions.openRgbDegraded), diagnostics: true },
};

export const DEFAULT_SCENARIO = "demo";
