/** Each context menu item id, with the visible control it mirrors; that control carries `data-mirror` with the same id. */
export const MIRRORS = {
  "connection.rescan": "Rescan devices in the connection panel",
  "connection.connect": "Connect to OpenRGB in the connection panel",
  "connection.demo": "Start demo mode in the connection panel",
  "connection.end": "Disconnect or Stop retrying in the connection panel",
  "profile-chip.save": "Save changes… in the profile panel",
  "profile-chip.open": "Open Profiles in the profile panel",
  "device.set-lighting": "Set lighting on the device page",
  "device.identify": "Identify device on the device page",
  "device.rename": "Rename device on the device page",
  "device.reported-name": "Use the reported name on the device page",
  "zone.identify": "Identify in the zone's row",
  "zone.rename": "Rename in the zone's row",
  "zone.reported-name": "Reported name in the zone's row",
} as const;

export type MirrorId = keyof typeof MIRRORS;
