import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupOutput } from "../../lib/api/bindings/SetupOutput";
import type { Names } from "../../lib/names";
import { hasDirectMode, reference } from "../devices/present";
import type { OutputChoice } from "./edits";
import { outputFor } from "./wiring";

export type Detected = {
  key: string;
  choice: OutputChoice;
  device: SessionDevice;
  title: string;
  leds: number;
  direct: boolean;
};

export function detected(session: SessionSnapshot, names: Names): Detected[] {
  return session.devices.flatMap((device): Detected[] => {
    const ref = reference(device);
    if (ref === null) {
      return [];
    }
    const zones = device.descriptor.zones;
    const direct = hasDirectMode(device);
    if (zones.length <= 1) {
      return [
        {
          key: `${ref}|all`,
          choice: { device: ref, zone: null, label: device.descriptor.name, zoneName: null },
          device,
          title: names.device(device),
          leds: device.descriptor.led_count,
          direct,
        },
      ];
    }
    return zones.map((zone, index) => ({
      key: `${ref}|${index}`,
      choice: { device: ref, zone: index, label: device.descriptor.name, zoneName: zone.name },
      device,
      title: `${names.zone(device, index)} on ${names.device(device)}`,
      leds: zone.led_count,
      direct,
    }));
  });
}

export function existing(setup: Setup, entry: Detected): SetupOutput | null {
  return outputFor(setup, { device: entry.choice.device, zone: entry.choice.zone });
}
