import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Menu, MenuItem } from "../../lib/shell/menu.svelte";
import type { AppState } from "../../lib/state/app.svelte";
import { applyingText } from "../effects/present";
import { identifyBlocked, reference } from "./present";

export function identifyReason(app: AppState, session: SessionSnapshot, device: SessionDevice): string | null {
  return applyingText(app.view) ?? identifyBlocked(session, device, app.identify?.kind === "running");
}

function naming(app: AppState, device: SessionDevice, zone: number | null): MenuItem[] {
  const ref = reference(device);
  if (ref === null || app.profiles.store === null) {
    return [];
  }
  const given = zone === null ? app.names.given(ref) : app.names.givenZone(ref, zone);
  const items: MenuItem[] = [
    {
      id: zone === null ? "device.rename" : "zone.rename",
      label: zone === null ? "Rename device" : "Rename zone",
      run: () => (app.naming = { device: ref, zone, action: "rename" }),
    },
  ];
  if (given !== null) {
    items.push({
      id: zone === null ? "device.reported-name" : "zone.reported-name",
      label: "Use the reported name",
      run: () => (app.naming = { device: ref, zone, action: "clear" }),
    });
  }
  return items;
}

export function deviceMenu(app: AppState, session: SessionSnapshot, key: string, device: SessionDevice): Menu {
  const ref = reference(device);
  const show = () => (app.selected = key);
  const items: MenuItem[] = [];
  if (ref !== null) {
    items.push({ id: "device.set-lighting", label: "Set lighting", run: () => (show(), app.setLighting(ref)) });
  }
  items.push({
    id: "device.identify",
    label: "Identify device",
    reason: identifyReason(app, session, device),
    run: () => (show(), ref !== null && app.identifyTarget({ device: ref, zone: null })),
  });
  const named = naming(app, device, null).map((item) => ({ ...item, run: (origin: HTMLElement) => (show(), item.run(origin)) }));
  return { label: `Actions for ${app.names.device(device)}`, items: [...items, ...named] };
}

export function zoneMenu(app: AppState, session: SessionSnapshot, device: SessionDevice, zone: number): Menu {
  const ref = reference(device);
  const identify: MenuItem = {
    id: "zone.identify",
    label: "Identify zone",
    reason: identifyReason(app, session, device),
    run: () => ref !== null && app.identifyTarget({ device: ref, zone }),
  };
  return { label: `Actions for zone ${app.names.zone(device, zone)}`, items: [identify, ...naming(app, device, zone)] };
}
