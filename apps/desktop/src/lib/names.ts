import type { DeviceNames } from "./api/bindings/DeviceNames";
import type { DeviceRef } from "./api/bindings/DeviceRef";
import type { SessionDevice } from "./api/bindings/SessionDevice";
import { reference } from "../features/devices/present";

/** The user's names for devices and zones; the reported name stands in where the user gave none. */
export class Names {
  private readonly byDevice: Map<DeviceRef, DeviceNames>;

  constructor(names: DeviceNames[] = []) {
    this.byDevice = new Map(names.map((entry) => [entry.device, entry]));
  }

  given(device: DeviceRef): string | null {
    return this.byDevice.get(device)?.name ?? null;
  }

  givenZone(device: DeviceRef, zone: number): string | null {
    return this.byDevice.get(device)?.zones.find((entry) => entry.zone === zone)?.name ?? null;
  }

  device(device: SessionDevice): string {
    const ref = reference(device);
    return (ref === null ? null : this.given(ref)) ?? device.descriptor.name;
  }

  zone(device: SessionDevice, zone: number): string {
    const ref = reference(device);
    const given = ref === null ? null : this.givenZone(ref, zone);
    return given ?? device.descriptor.zones[zone]?.name ?? `zone ${zone + 1}`;
  }
}
