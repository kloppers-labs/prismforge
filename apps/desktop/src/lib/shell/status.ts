import type { SessionSnapshot } from "../api/bindings/SessionSnapshot";
import type { ShellView } from "../api/bindings/ShellView";
import type { StoreView } from "../api/bindings/StoreView";
import { isSimulated, place, summarise, type Tone } from "../../features/connection/wording";
import { isLive } from "../../features/devices/present";
import { applyingText } from "../../features/effects/present";
import { onConnectText } from "../../features/profiles/present";

export type StatusTone = Tone | "idle";

export type ConnectionStep = "none" | "ended" | "starting" | "connecting" | "retrying" | "live";

export type ConnectionStatus = {
  step: ConnectionStep;
  tone: StatusTone;
  label: string;
  where: string | null;
  lines: string[];
  demo: boolean;
};

export function ended(session: SessionSnapshot | null): boolean {
  const connection = session?.connection;
  return typeof connection === "object" && "Disconnected" in connection && connection.Disconnected.reason !== null;
}

export function connectionStatus(session: SessionSnapshot | null): ConnectionStatus {
  if (session === null) {
    return {
      step: "none",
      tone: "idle",
      label: "Not connected",
      where: null,
      lines: ["PrismForge is not connected. Connect to OpenRGB on the Devices screen, or start demo mode."],
      demo: false,
    };
  }
  const summary = summarise(session);
  const demo = isSimulated(session);
  const where = demo ? "Demo backend" : session.backend.address ? `OpenRGB ${session.backend.address}` : "OpenRGB";
  const base = { tone: summary.tone, label: summary.label, where, demo };
  const note = demo ? ["Demo mode: these devices are simulated. Nothing is sent to hardware."] : [];
  if (ended(session)) {
    return { ...base, step: "ended", lines: ["The connection stopped. PrismForge does not try again by itself.", ...summary.reasons, ...note] };
  }
  if (isLive(session)) {
    const lead = summary.reasons.length > 0 ? `Connected to ${place(session)}, with problems.` : `Connected to ${place(session)}.`;
    return { ...base, step: "live", lines: [lead, ...summary.reasons, ...note] };
  }
  if (summary.retrying) {
    return { ...base, step: "retrying", lines: [...summary.reasons, ...note] };
  }
  if (session.connection === "Connecting") {
    return { ...base, step: "connecting", lines: [...summary.reasons, ...note] };
  }
  return { ...base, step: "starting", lines: [`PrismForge is starting the connection to ${place(session)}.`, ...note] };
}

export function connectionName(status: ConnectionStatus): string {
  return status.where ? `Connection: ${status.label}, ${status.where}` : `Connection: ${status.label}`;
}

export type ProfileStatus = {
  title: string;
  detail: string | null;
  name: string;
  lines: string[];
  busy: boolean;
  saveable: string | null;
};

function unsavedDetail(unsaved: number | null): string | null {
  if (unsaved === null) {
    return "Unsaved changes unknown";
  }
  if (unsaved === 0) {
    return null;
  }
  return unsaved === 1 ? "1 unsaved change" : `${unsaved} unsaved changes`;
}

function unsavedLine(unsaved: number | null): string {
  if (unsaved === null) {
    return "PrismForge cannot tell now whether it has unsaved changes.";
  }
  if (unsaved === 0) {
    return "It has no unsaved changes.";
  }
  return unsaved === 1 ? "It has 1 unsaved change." : `It has ${unsaved} unsaved changes.`;
}

function markLines(view: ShellView): string[] {
  const state = view.on_connect;
  if (typeof state === "object" && ("Armed" in state || "Unreadable" in state)) {
    return [onConnectText(state, view.session)];
  }
  return [];
}

/** `null` while the store of a running session is still being read, so nothing is claimed early. */
export function profileStatus(view: ShellView, store: StoreView | null, marked: string | null = null): ProfileStatus | null {
  const applying = view.applying;
  if (applying) {
    return {
      title: `Applying ${applying.name}`,
      detail: applying.automatic ? "on connect" : null,
      name: `Profile: applying ${applying.name}${applying.automatic ? " on connect" : ""}`,
      lines: [applyingText(view) ?? ""],
      busy: true,
      saveable: null,
    };
  }
  const summary = store?.summary;
  const active = summary?.profiles.find((profile) => profile.id === summary.active) ?? null;
  if (active && summary) {
    const detail = unsavedDetail(summary.unsaved_changes);
    const live = view.session !== null && isLive(view.session);
    return {
      title: active.name,
      detail,
      name: `Profile: ${active.name}, ${detail ? detail.toLowerCase() : "no unsaved changes"}`,
      lines: [`${active.name} is the active profile in this session.`, unsavedLine(summary.unsaved_changes), ...markLines(view)],
      busy: false,
      saveable: live && summary.unsaved_changes !== 0 ? active.id : null,
    };
  }
  if (store === null && view.session !== null && !ended(view.session)) {
    return null;
  }
  const idle = view.session === null || ended(view.session);
  const armed = typeof view.on_connect === "object" && "Armed" in view.on_connect ? view.on_connect.Armed.name : idle ? marked : null;
  if (view.session === null && armed === null) {
    return null;
  }
  const detail = armed ? `${armed} applies on connect` : null;
  return {
    title: "No active profile",
    detail,
    name: detail ? `Profile: none active, ${detail}` : "Profile: none active",
    lines: [
      "No profile is active. A profile becomes active when you apply it or save it while PrismForge is connected.",
      ...(idle && armed !== null
        ? [`When you connect to OpenRGB, PrismForge applies ${armed} as soon as it has read the devices.`]
        : markLines(view)),
    ],
    busy: false,
    saveable: null,
  };
}
