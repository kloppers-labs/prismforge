import type { AppState } from "../state/app.svelte";
import type { Menu, MenuItem } from "./menu.svelte";
import type { ConnectionStatus, ProfileStatus } from "./status";

export function busyText(app: AppState): string {
  return app.working === "refresh" ? "PrismForge is still reading the device list." : "PrismForge is still working on an earlier request.";
}

/** Runs a strip action the way its panel does: a covering drawer closes first. */
function strip(app: AppState, action: () => unknown): () => void {
  return () => {
    app.uncover();
    void action();
  };
}

export function connectionMenu(app: AppState, status: ConnectionStatus, onconnect: () => void): Menu {
  const busy = app.working === null ? null : busyText(app);
  const items: MenuItem[] = [];
  if (status.step === "live") {
    items.push({ id: "connection.rescan", label: "Rescan devices", reason: busy, run: strip(app, () => app.refresh()) });
  } else if (status.step === "none" || status.step === "ended") {
    items.push(
      { id: "connection.connect", label: "Connect to OpenRGB", run: strip(app, onconnect) },
      { id: "connection.demo", label: "Start demo mode", reason: busy, run: strip(app, () => app.startDemo()) },
    );
  }
  if (status.step !== "none" && status.step !== "ended") {
    items.push({
      id: "connection.end",
      label: status.step === "retrying" ? "Stop retrying" : "Disconnect",
      reason: app.working === "disconnect" ? "PrismForge is disconnecting now." : null,
      danger: true,
      run: strip(app, () => app.disconnect()),
    });
  }
  return { label: "Connection actions", items, focus: items.length === 1 && items[0]?.danger ? "menu" : "first" };
}

export function profileChipMenu(app: AppState, status: ProfileStatus): Menu {
  const items: MenuItem[] = [];
  const saveable = status.saveable;
  if (saveable) {
    items.push({ id: "profile-chip.save", label: `Save changes to ${status.title}`, run: strip(app, () => app.saveActiveChanges(saveable)) });
  }
  items.push({ id: "profile-chip.open", label: "Open Profiles", run: strip(app, () => app.openProfiles()) });
  return { label: "Profile actions", items };
}
