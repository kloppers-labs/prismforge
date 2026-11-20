import type { AutomationView } from "../lib/api/bindings/AutomationView";
import type { GroupsView } from "../lib/api/bindings/GroupsView";
import * as automations from "../test/fixtures/automations";
import type { SetupStoreView } from "../lib/api/bindings/SetupStoreView";
import { acrossDemoSkipped } from "../test/fixtures/acrossReports";
import { liveRealAcross } from "../test/fixtures/live";
import { setupDemoEmpty, setupDemoTower, setupDemoZoneChanged, setupRealEmpty, setupRealTower } from "../test/fixtures/setups";
import type { LifecycleView } from "../lib/api/bindings/LifecycleView";
import type { LiveOutput } from "../lib/api/bindings/LiveOutput";
import { batchDuringIdentify } from "../test/fixtures/batches";
import { groupsDemo, groupsRealMissing } from "../test/fixtures/groups";
import { liveKeyboardAndStrip } from "../test/fixtures/live";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../lib/api/bindings/ShellView";
import type { StoreView } from "../lib/api/bindings/StoreView";
import { isLive } from "../features/devices/present";
import { lifecycleChosen, lifecycleFresh, lifecycleStartPath, lifecycleTrayStopped } from "../test/fixtures/lifecycles";
import * as sessions from "../test/fixtures/sessions";
import { storeEmpty, storeOffline, storeRecovered, storeSaved, storeTwo, storeUnsaved } from "../test/fixtures/stores";
import { viewApplying, viewArmedRetrying, viewConnectApplied } from "../test/fixtures/views";
import type { Handler } from "./transport";

export type Area = "Devices" | "Effects" | "Setup" | "Profiles" | "Automation" | "Settings";

export type Scenario = {
  label: string;
  view: ShellView;
  store: StoreView;
  lifecycle: LifecycleView;
  area?: Area;
  diagnostics?: boolean;
  handlers?: Record<string, Handler>;
  groups?: GroupsView;
  /** Targets checked on Effects, by their row key `device|zone` or `device|all`. */
  select?: string[];
  group?: string;
  /** A group whose Members and actions are opened. */
  groupOpen?: string;
  effect?: string;
  apply?: boolean;
  stage?: "live";
  live?: LiveOutput;
  rescan?: boolean;
  setup?: SetupStoreView;
  automation?: AutomationView;
  /** The rule the Automation screen scrolls to, by its trigger in lower case. */
  rule?: string;
  /** What the Setup area shows: a tab, a template added, components selected, an effect drafted, applied or live. */
  setupTab?: "PC" | "Desk";
  setupTemplate?: string;
  setupSelect?: string[];
  setupEffect?: string;
  setupApply?: boolean;
  scrollTo?: string;
};

function onConnect(session: SessionSnapshot | null): ShellView["on_connect"] {
  if (session === null) {
    return "NotApplicable";
  }
  return isLive(session) ? { Fired: { ran: null, next: null } } : "NotArmed";
}

function view(session: SessionSnapshot | null, extra: Partial<ShellView> = {}): ShellView {
  return { revision: 1, session, applying: null, connect_apply: null, on_connect: onConnect(session), rule_run: null, ...extra };
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

/** The demo batch session with the second memory module at a faster speed, so its settings differ. */
function mixedValues(): SessionSnapshot {
  const session = structuredClone(sessions.demoBatchRunning);
  const memory = session.output.scene.assignments.find((assignment) => assignment.target.device === "demo-memory-2");
  if (memory) {
    memory.effect.values = { ...memory.effect.values, speed: { Number: 60 } };
  }
  return session;
}

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
  "effects-several-mixed": {
    ...base,
    label: "Effects, several targets with a mixed device",
    view: view(sessions.demoConnected),
    area: "Effects",
    groups: groupsDemo,
    select: ["demo-graphics-card|all", "demo-motherboard|0", "demo-motherboard|2", "demo-memory-2|all"],
    effect: "Rainbow wave",
  },
  "effects-group-missing": {
    ...openRgb,
    label: "Effects, a group with a member not found today",
    view: view(sessions.openRgbShelfGone),
    area: "Effects",
    groups: groupsRealMissing,
    group: "Shelf and desk",
  },
  "effects-batch-skips": {
    ...base,
    label: "Effects, a batch result with a skipped target",
    view: view(sessions.demoIdentifyingGpu),
    area: "Effects",
    groups: groupsDemo,
    select: ["demo-memory-1|all", "demo-memory-2|all", "demo-motherboard|all", "demo-graphics-card|all"],
    effect: "Rainbow wave",
    apply: true,
    handlers: { apply_effect_batch: () => batchDuringIdentify },
  },
  "effects-live": {
    ...openRgb,
    label: "Effects, the live output running",
    view: view(sessions.openRgbKeyboardRunning),
    area: "Effects",
    select: [`${liveKeyboardAndStrip.devices[0]?.device}|all`, `${liveKeyboardAndStrip.devices[1]?.device}|all`],
    stage: "live",
    live: liveKeyboardAndStrip,
  },
  "effects-mixed-values": {
    ...base,
    label: "Effects, several targets running one effect with different values",
    view: view(mixedValues()),
    area: "Effects",
    select: ["demo-graphics-card|all", "demo-memory-2|all"],
  },
  "effects-group-apply": {
    ...base,
    label: "Effects, apply the chosen effect to a group",
    view: view(sessions.demoConnected),
    area: "Effects",
    groups: groupsDemo,
    select: ["demo-memory-1|all"],
    effect: "Rainbow wave",
    groupOpen: "Desk",
  },
  "effects-draft": {
    ...base,
    label: "Effects, the draft preview",
    view: view(sessions.demoConnected),
    area: "Effects",
    select: ["demo-graphics-card|all", "demo-motherboard|1"],
    effect: "Gradient",
  },
};

/** The demo tower with a case outline, the graphics card as a block and the board edge as a free line. */
function demoLibrary(): SetupStoreView {
  const view = structuredClone(setupDemoTower);
  const pc = view.setup.views.find((each) => each.id === "v-pc")!;
  pc.outlines = [{ id: "u-1", name: "Case", kind: "Case", x: 535, y: 350, width: 900, height: 640, rotation: 0 }];
  const graphics = view.setup.components.find((component) => component.id === "c-graphics")!;
  graphics.frame = { width: 112, height: 40 };
  const edge = view.setup.components.find((component) => component.id === "c-board-edge")!;
  edge.geometry = { Line: { leds: 6, points: [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 60 }], first: "AtStart" } };
  return view;
}

/** The real tower whose desk view also draws the desk, the monitor and the PC. */
function realDesk(): SetupStoreView {
  const view = structuredClone(setupRealTower);
  const desk = view.setup.views.find((each) => each.id === "v-desk")!;
  desk.outlines = [
    { id: "u-1", name: "Desk", kind: "Desk", x: 500, y: 660, width: 920, height: 40, rotation: 0 },
    { id: "u-2", name: "Monitor", kind: "Monitor", x: 430, y: 200, width: 460, height: 280, rotation: 0 },
    { id: "u-3", name: "PC", kind: "Pc", x: 840, y: 300, width: 170, height: 300, rotation: 0 },
  ];
  return view;
}

Object.assign(SCENARIOS, {
  "setup-empty": { ...openRgb, label: "Setup, an empty PC view", view: view(sessions.openRgbSetupDevices), area: "Setup", setup: setupRealEmpty },
  "setup-template": {
    ...base,
    label: "Setup, the Tower PC template being mapped",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: setupDemoEmpty,
    setupTemplate: "Tower PC",
    scrollTo: "Not wired yet",
  },
  "setup-live": {
    ...openRgb,
    label: "Setup, a finished PC view with live colours",
    view: view(sessions.openRgbAcrossRunning),
    area: "Setup",
    setup: setupRealTower,
    stage: "live",
    live: liveRealAcross,
  },
  "setup-mirror": {
    ...base,
    label: "Setup, a mirrored fan hub and a two-part cooler",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: setupDemoTower,
    setupSelect: ["c-front-fan-2"],
    scrollTo: "Outputs",
  },
  "setup-desk": {
    ...openRgb,
    label: "Setup, the desk view",
    view: view(sessions.openRgbAcrossRunning),
    area: "Setup",
    setup: realDesk(),
    setupTab: "Desk",
  },
  "setup-library": {
    ...base,
    label: "Setup, a case outline, a block and a free line",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: demoLibrary(),
    setupSelect: ["c-graphics"],
  },
  "setup-points": {
    ...base,
    label: "Setup, the points of a free line edited with number fields",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: demoLibrary(),
    setupSelect: ["c-board-edge"],
    scrollTo: "Points of the line",
  },
  "setup-draft": {
    ...base,
    label: "Setup, the draft preview across the setup",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: setupDemoTower,
    setupEffect: "rainbow-wave",
  },
  "setup-excluded": {
    ...base,
    label: "Setup, an apply with outputs that got nothing",
    view: view(sessions.demoAcrossSkipped),
    area: "Setup",
    setup: setupDemoZoneChanged,
    setupEffect: "rainbow-wave",
    setupApply: true,
    handlers: { apply_across: () => acrossDemoSkipped },
    scrollTo: "Results for each target",
  },
  "setup-list": {
    ...base,
    label: "Setup, the list twin with two components selected",
    view: view(sessions.demoConnected),
    area: "Setup",
    setup: setupDemoTower,
    setupSelect: ["c-cooler-left", "c-cooler-right"],
  },
});

export const DEFAULT_SCENARIO = "demo";

Object.assign(SCENARIOS, {
  "automation-off": { ...base, label: "Automation, no rule on", view: view(sessions.demoConnected), area: "Automation", automation: automations.demoNoRules },
  "automation-sleep-wake": {
    ...openRgb,
    label: "Automation, lights off at sleep and a profile at wake",
    view: view(sessions.openRgbConnected),
    area: "Automation",
    automation: automations.openRgbSleepAndWake,
  },
  "automation-last-runs": {
    ...openRgb,
    label: "Automation, last runs with a skipped device",
    view: view(sessions.openRgbConnected),
    area: "Automation",
    automation: automations.openRgbAfterSleepAndWake,
    rule: "wakes",
  },
  "automation-deleted-profile": {
    ...openRgb,
    label: "Automation, a rule whose profile was deleted",
    view: view(sessions.openRgbConnected),
    area: "Automation",
    automation: automations.openRgbDeletedProfile,
    rule: "wakes",
  },
  "automation-openrgb-also": {
    ...openRgb,
    label: "Automation, OpenRGB also acts",
    view: view(sessions.openRgbConnected),
    area: "Automation",
    automation: automations.openRgbAlsoActs,
    rule: "sleeps",
  },
  "automation-not-here": {
    ...base,
    label: "Automation on macOS or Linux",
    view: view(sessions.demoConnected),
    area: "Automation",
    automation: automations.demoNotOnThisSystem,
    rule: "sleeps",
  },
  "automation-migrated": {
    ...openRgb,
    label: "Automation, the connects rule from an earlier mark",
    view: view(sessions.openRgbConnected),
    area: "Automation",
    automation: automations.openRgbMigratedConnects,
  },
} satisfies Record<string, Scenario>);
