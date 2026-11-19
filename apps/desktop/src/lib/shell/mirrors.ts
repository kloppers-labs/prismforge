/** Each context menu item id, with the visible control it mirrors; that control carries `data-mirror` with the same id. */
export const MIRRORS = {
  "connection.rescan": "Rescan devices in the connection panel",
  "connection.connect": "Connect to OpenRGB in the connection panel",
  "connection.demo": "Start demo mode in the connection panel",
  "connection.end": "Disconnect or Stop retrying in the connection panel",
  "profile-chip.save": "Save changes… in the profile panel",
  "profile-chip.open": "Open Profiles in the profile panel",
} as const;

export type MirrorId = keyof typeof MIRRORS;
