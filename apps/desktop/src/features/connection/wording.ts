import type { LossCause } from "../../lib/api/bindings/LossCause";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { StateReason } from "../../lib/api/bindings/StateReason";

export type Tone = "ok" | "working" | "warning" | "problem";

export type ConnectionSummary = {
  tone: Tone;
  label: string;
  reasons: string[];
  retrying: boolean;
};

export function isSimulated(session: SessionSnapshot): boolean {
  return session.backend.origin === "Simulated";
}

export function serverName(session: SessionSnapshot): string {
  return isSimulated(session) ? "the demo backend" : "OpenRGB";
}

export function place(session: SessionSnapshot): string {
  if (isSimulated(session)) {
    return "the demo backend";
  }
  return session.backend.address ? `the OpenRGB server at ${session.backend.address}` : "the OpenRGB server";
}

export function summarise(session: SessionSnapshot): ConnectionSummary {
  const connection = session.connection;
  const where = place(session);
  if (connection === "Connected") {
    return { tone: "ok", label: "Connected", reasons: [], retrying: false };
  }
  if (connection === "Connecting") {
    return { tone: "working", label: "Connecting", reasons: [`Connecting to ${where}.`], retrying: false };
  }
  if ("Retrying" in connection) {
    const { attempt, reason } = connection.Retrying;
    return {
      tone: "problem",
      label: `Retrying, attempt ${attempt}`,
      reasons: [
        `Attempt ${attempt} failed. ${explain(reason, where)}`,
        "PrismForge keeps trying on its own, with a longer wait after each attempt (at most 30 seconds).",
      ],
      retrying: true,
    };
  }
  if ("Degraded" in connection) {
    return {
      tone: "warning",
      label: "Connected with problems",
      reasons: connection.Degraded.reasons.map((reason) => explain(reason, where)),
      retrying: false,
    };
  }
  const reason = connection.Disconnected.reason;
  return {
    tone: reason ? "problem" : "working",
    label: reason ? "Not connected" : "Starting",
    reasons: reason ? [explain(reason, where)] : [],
    retrying: false,
  };
}

export function explain(reason: StateReason, where: string): string {
  if (reason === "ServerUnreachable") {
    return `Nothing answered at ${where}.`;
  }
  if (reason === "InventoryStale") {
    return "The device list changed. PrismForge is reading it again.";
  }
  if ("UnsupportedProtocol" in reason) {
    const version = reason.UnsupportedProtocol.server_version;
    if (version === null) {
      return `${capitalise(where)} did not answer the version request. It may still be starting, or it may have stopped responding.`;
    }
    return `${capitalise(where)} uses SDK protocol ${version}. PrismForge needs protocol 6, which comes with OpenRGB 1.0 and later.`;
  }
  if ("ConnectionLost" in reason) {
    return `The connection was lost: ${lossText(reason.ConnectionLost.cause)}`;
  }
  if ("ControllerDataRejected" in reason) {
    const count = reason.ControllerDataRejected.controllers;
    return `${count === 1 ? "One controller" : `${count} controllers`} sent data PrismForge could not read. The other devices still work.`;
  }
  const count = reason.AmbiguousDevices.devices;
  return `${count} devices look identical to another device. PrismForge will not light them until it can tell them apart.`;
}

function lossText(cause: LossCause): string {
  switch (cause) {
    case "Timeout":
      return "the server stopped answering in time.";
    case "ServerClosed":
      return "the server closed the connection.";
    case "Transport":
      return "the network connection failed.";
    case "ProtocolViolation":
      return "the server sent data PrismForge could not read.";
    case "CallAbandoned":
      return "an earlier request was cut off, so the connection was reset.";
  }
}

export function capitalise(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}
