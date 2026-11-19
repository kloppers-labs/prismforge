import type { DeviceKind } from "../../lib/api/bindings/DeviceKind";
import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { ZoneDescriptor } from "../../lib/api/bindings/ZoneDescriptor";

const KINDS: Record<DeviceKind, string> = {
  Motherboard: "Motherboard",
  Memory: "Memory module",
  Gpu: "Graphics card",
  Cooler: "Cooler",
  LedStrip: "LED strip",
  Keyboard: "Keyboard",
  Mouse: "Mouse",
  Mousemat: "Mouse mat",
  Headset: "Headset",
  HeadsetStand: "Headset stand",
  Gamepad: "Gamepad",
  Light: "Light",
  Speaker: "Speaker",
  Virtual: "Virtual device",
  Storage: "Storage drive",
  Case: "Case",
  Microphone: "Microphone",
  Accessory: "Accessory",
  Keypad: "Keypad",
  Laptop: "Laptop",
  Monitor: "Monitor",
  Unknown: "Unknown device type",
};

export function kindLabel(kind: DeviceKind): string {
  return KINDS[kind];
}

export function deviceKey(device: SessionDevice, index: number): string {
  const identity = device.identity;
  if ("Matched" in identity) {
    return identity.Matched;
  }
  if ("New" in identity) {
    return identity.New;
  }
  return `identical-${index}`;
}

export function reference(device: SessionDevice): string | null {
  const identity = device.identity;
  if ("Matched" in identity) {
    return identity.Matched;
  }
  if ("New" in identity) {
    return identity.New;
  }
  return null;
}

export function isAmbiguous(device: SessionDevice): boolean {
  return "Ambiguous" in device.identity;
}

export function hasDirectMode(device: SessionDevice): boolean {
  return device.descriptor.capabilities.direct_mode !== null;
}

export function isLive(session: SessionSnapshot): boolean {
  const connection = session.connection;
  return connection === "Connected" || (typeof connection === "object" && "Degraded" in connection);
}

export function identifyBlocked(
  session: SessionSnapshot,
  device: SessionDevice,
  identifyRunning: boolean,
): string | null {
  if (isAmbiguous(device)) {
    return "PrismForge cannot tell this device apart from an identical one.";
  }
  if (!hasDirectMode(device)) {
    return "This device has no Direct Mode, so PrismForge cannot light it.";
  }
  if (!device.available) {
    return isLive(session)
      ? "OpenRGB does not report this device right now."
      : "Not connected. Identify works again after PrismForge reconnects.";
  }
  if (identifyRunning || session.identify !== null) {
    return "Another identify is running. Wait until it finishes.";
  }
  return null;
}

export function zoneShape(zone: ZoneDescriptor): string {
  const kind = zone.kind;
  if (zone.matrix) {
    return `Grid, ${zone.matrix.width} by ${zone.matrix.height}`;
  }
  if (typeof kind === "object") {
    return `Other (type ${kind.Other})`;
  }
  switch (kind) {
    case "Single":
      return "Single light";
    case "Linear":
      return "Line";
    case "Matrix":
      return "Grid";
    case "LinearLoop":
      return "Ring";
    case "MatrixLoopX":
    case "MatrixLoopY":
      return "Grid, wraps around";
    case "Segmented":
      return "Segments";
  }
}

export function zoneSize(zone: ZoneDescriptor): string {
  if (zone.leds_min === zone.leds_max) {
    return "Fixed size";
  }
  return `Resizable, ${zone.leds_min} to ${zone.leds_max} LEDs`;
}

export function ledCount(count: number): string {
  return count === 1 ? "1 LED" : `${count} LEDs`;
}

export function distinction(devices: SessionDevice[], name: (device: SessionDevice) => string): (string | null)[] {
  return devices.map((device) => {
    const twins = devices.filter((other) => name(other) === name(device));
    if (twins.length < 2) {
      return null;
    }
    const distinct = (value: (each: SessionDevice) => string) => {
      const values = twins.map(value);
      return values.every((each) => each.trim().length > 0) && new Set(values).size === values.length;
    };
    if (distinct((each) => each.descriptor.location)) {
      return device.descriptor.location;
    }
    if (distinct((each) => each.descriptor.serial)) {
      return `Serial ending ${device.descriptor.serial.slice(-4)}`;
    }
    return `${twins.indexOf(device) + 1} of ${twins.length}`;
  });
}
