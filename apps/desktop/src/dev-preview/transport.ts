import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";
import { LIFECYCLE_EVENT, LIVE_EVENT, VIEW_EVENT } from "../lib/api/client";
import type { LiveOutput } from "../lib/api/bindings/LiveOutput";
import type { LifecycleView } from "../lib/api/bindings/LifecycleView";
import type { AcrossPreview } from "../lib/api/bindings/AcrossPreview";
import type { AcrossPreviewRequest } from "../lib/api/bindings/AcrossPreviewRequest";
import type { PreviewBatch } from "../lib/api/bindings/PreviewBatch";
import { viewTargets } from "../features/setup/colors";
import { span } from "../features/setup/wiring";
import type { PreviewRequest } from "../lib/api/bindings/PreviewRequest";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../lib/api/bindings/ShellView";
import { catalog } from "../test/fixtures/catalog";
import { groupsEmpty } from "../test/fixtures/groups";
import { setupDemoEmpty } from "../test/fixtures/setups";
import { detailsEvening, detailsImported, detailsOffline } from "../test/fixtures/profileDetails";
import { demoConnected, openRgbConnecting } from "../test/fixtures/sessions";
import type { AutomationView } from "../lib/api/bindings/AutomationView";
import type { RuleAction } from "../lib/api/bindings/RuleAction";
import { demoNoRules } from "../test/fixtures/automations";
import type { Scenario } from "./scenarios";

export type Handler = (args: Record<string, unknown>) => unknown;

const details: Record<string, unknown> = {
  "p-0000000000000001": detailsEvening,
  "p-0000000000000002": detailsImported,
  "p-0000000000000003": detailsOffline,
};

function hue(at: number): { r: number; g: number; b: number } {
  const channel = (offset: number) => Math.round(127.5 + 127.5 * Math.cos(2 * Math.PI * (at - offset)));
  return { r: channel(0), g: channel(1 / 3), b: channel(2 / 3) };
}

/** Pixel values are made up here; the component that draws them is the real one. */
function preview(args: Record<string, unknown>): PreviewBatch {
  const request = args.request as PreviewRequest;
  const frames = Array.from({ length: request.frame_count }, (_, frame) =>
    Array.from({ length: request.led_count }, (_, led) => hue(led / Math.max(request.led_count, 1) + frame / 50)),
  );
  return { led_count: request.led_count, start_ms: request.start_ms, interval_ms: request.interval_ms, is_static: false, frames };
}

/** Made-up colours for each target of the drawing; the drawing and the mapping to components are the real ones. */
function previewAcross(args: Record<string, unknown>): AcrossPreview {
  const request = args.request as AcrossPreviewRequest;
  const setup = request.setup;
  const targets = viewTargets(setup, request.view).map((target) => {
    const outputs = setup.outputs.filter((output) => output.device === target.device && output.zone === target.zone);
    const leds = Math.max(0, ...outputs.map((output) => output.first_led + span(setup, output)));
    const frames = Array.from({ length: request.frame_count }, (_, frame) =>
      Array.from({ length: leds }, (_, led) => hue((target.zone ?? 3) / 7 + led / Math.max(leds, 1) / 3 + frame / 50)),
    );
    return { target, frames };
  });
  return { start_ms: request.start_ms, interval_ms: request.interval_ms, is_static: false, targets };
}

/** Moves each zone's colours on by one LED per frame, so the live view animates as a real feed would. */
function liveFeed(live: LiveOutput | undefined) {
  let timer: ReturnType<typeof setInterval> | undefined;
  return (args: Record<string, unknown>) => {
    clearInterval(timer);
    const targets = args.targets as unknown[];
    if (!live || targets.length === 0) {
      return null;
    }
    let step = 0;
    timer = setInterval(() => {
      step += 1;
      const frame: LiveOutput = {
        devices: live.devices.map((device) => ({
          ...device,
          zones: device.zones.map((zone) => ({ ...zone, colors: zone.colors.map((_, index) => zone.colors[(index + step) % zone.colors.length]!) })),
        })),
      };
      void emit(LIVE_EVENT, frame);
    }, 40);
    return null;
  };
}

/** Keeps the rule change in the page only; the real store and its checks are in Rust. */
function rules(start: AutomationView) {
  let current = structuredClone(start);
  return {
    view: () => current,
    set: (args: Record<string, unknown>) => {
      const action = args.action as RuleAction;
      const profile = typeof action === "object" ? current.profiles.find((choice) => choice.id === action.ApplyProfile.profile) : undefined;
      current = {
        ...current,
        rules: current.rules.map((rule) =>
          rule.trigger === args.trigger
            ? { ...rule, enabled: args.enabled as boolean, action, profile: profile ? { Ready: { name: profile.name } } : null, problem: null }
            : rule,
        ),
      };
      return current;
    },
  };
}

/** Stands in for the Tauri transport so the real App runs in a browser; nothing reaches Rust or hardware. */
export function installTransport(scenario: Scenario) {
  let revision = scenario.view.revision;
  const view = (session: SessionSnapshot | null): ShellView => ({ ...scenario.view, revision: ++revision, session, applying: null, connect_apply: null });
  const push = (next: ShellView) => {
    void emit(VIEW_EVENT, next);
    return next;
  };
  const automation = rules(scenario.automation ?? demoNoRules);
  const defaults: Record<string, Handler> = {
    view: () => scenario.view,
    automation: automation.view,
    set_rule: automation.set,
    mark_runs_seen: automation.view,
    lifecycle: () => scenario.lifecycle,
    profile_store: () => scenario.store,
    profile_details: (args) => details[String(args.id)] ?? Promise.reject({ Profile: "NotFound" }),
    effect_catalog: () => catalog,
    groups: () => scenario.groups ?? { ...groupsEmpty, kind: scenario.store.kind },
    setup: () => scenario.setup ?? { ...setupDemoEmpty, kind: scenario.store.kind },
    save_setup: (args) => ({ ...(scenario.setup ?? setupDemoEmpty), setup: args.setup }),
    watch_live_output: liveFeed(scenario.live),
    preview_effect: preview,
    preview_across: previewAcross,
    refresh: () => null,
    connect_openrgb: () => push(view(openRgbConnecting)),
    start_demo: () => push(view(demoConnected)),
    disconnect: () => push(view(null)),
  };
  mockIPC(
    (command, args) => {
      const handler = scenario.handlers?.[command] ?? defaults[command];
      if (!handler) {
        return Promise.reject(`the interface preview does not simulate ${command}`);
      }
      return handler((args ?? {}) as Record<string, unknown>);
    },
    { shouldMockEvents: true },
  );
  return {
    lifecycle: (next: LifecycleView) => emit(LIFECYCLE_EVENT, next),
  };
}
