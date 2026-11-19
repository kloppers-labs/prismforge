import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";
import { LIFECYCLE_EVENT, LIVE_EVENT, VIEW_EVENT } from "../lib/api/client";
import type { LiveOutput } from "../lib/api/bindings/LiveOutput";
import type { LifecycleView } from "../lib/api/bindings/LifecycleView";
import type { PreviewBatch } from "../lib/api/bindings/PreviewBatch";
import type { PreviewRequest } from "../lib/api/bindings/PreviewRequest";
import type { SessionSnapshot } from "../lib/api/bindings/SessionSnapshot";
import type { ShellView } from "../lib/api/bindings/ShellView";
import { catalog } from "../test/fixtures/catalog";
import { groupsEmpty } from "../test/fixtures/groups";
import { detailsEvening, detailsImported, detailsOffline } from "../test/fixtures/profileDetails";
import { demoConnected, openRgbConnecting } from "../test/fixtures/sessions";
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

/** Stands in for the Tauri transport so the real App runs in a browser; nothing reaches Rust or hardware. */
export function installTransport(scenario: Scenario) {
  let revision = scenario.view.revision;
  const view = (session: SessionSnapshot | null): ShellView => ({ ...scenario.view, revision: ++revision, session, applying: null, connect_apply: null });
  const push = (next: ShellView) => {
    void emit(VIEW_EVENT, next);
    return next;
  };
  const defaults: Record<string, Handler> = {
    view: () => scenario.view,
    lifecycle: () => scenario.lifecycle,
    profile_store: () => scenario.store,
    profile_details: (args) => details[String(args.id)] ?? Promise.reject({ Profile: "NotFound" }),
    effect_catalog: () => catalog,
    groups: () => scenario.groups ?? { ...groupsEmpty, kind: scenario.store.kind },
    watch_live_output: liveFeed(scenario.live),
    preview_effect: preview,
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
