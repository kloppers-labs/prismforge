import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { OutputId } from "../../lib/api/bindings/OutputId";
import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupOutput } from "../../lib/api/bindings/SetupOutput";
import { reference } from "../devices/present";
import { ledCount } from "./geometry";
import { span } from "./wiring";

/** A reason the store would refuse the setup now; `component` is null when the parts together do not fit. */
export type Problem = { component: ComponentId | null; output: OutputId; text: string };

export type Capacity = { name: string; max: number; available: number };

export function deviceFor(session: SessionSnapshot, output: SetupOutput): SessionDevice | null {
  return session.devices.find((device) => reference(device) === output.device) ?? null;
}

/** The most LEDs the backend allows on the output and how many it has now; null when the output does not match a reported zone. */
export function capacity(session: SessionSnapshot, output: SetupOutput): Capacity | null {
  const device = deviceFor(session, output);
  if (!device) {
    return null;
  }
  if (output.zone === null) {
    const max = device.descriptor.zones.reduce((sum, zone) => sum + zone.leds_max, 0);
    return { name: device.descriptor.name, max, available: device.descriptor.led_count };
  }
  const zone = device.descriptor.zones[output.zone];
  if (!zone || (output.zone_name !== null && output.zone_name !== zone.name)) {
    return null;
  }
  return { name: zone.name, max: zone.leds_max, available: zone.led_count };
}

/** The same checks as the store's save, per output of a device the session reports, so the reason shows before Save. */
export function problems(setup: Setup, session: SessionSnapshot): Problem[] {
  return setup.outputs.flatMap((output) => {
    const room = capacity(session, output);
    if (!room) {
      return [];
    }
    const found: Problem[] = [];
    for (const id of output.branches.flat()) {
      const component = setup.components.find((each) => each.id === id);
      const leds = component ? ledCount(component.geometry) : 0;
      if (component && leds > room.max) {
        found.push({ component: id, output: output.id, text: `${component.name} has ${leds} LEDs. ${room.name} takes at most ${room.max}.` });
      }
    }
    const needed = output.first_led + span(setup, output);
    if (needed > room.available) {
      const resize = room.max > room.available ? `, or resize the zone in OpenRGB first; it takes up to ${room.max}` : "";
      found.push({ component: null, output: output.id, text: `The parts on ${room.name} need ${needed} LEDs, and it has ${room.available}. Use fewer LEDs${resize}.` });
    }
    return found;
  });
}
