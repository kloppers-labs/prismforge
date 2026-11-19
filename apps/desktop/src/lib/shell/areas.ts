import type { Screen } from "../state/app.svelte";
import type { IconName } from "./Icon.svelte";

export type Area = { screen: Screen; label: string; icon: IconName; key: string };

/** The rail's main group in order; Ctrl (Command on macOS) with the key opens the area. */
export const MAIN_AREAS: readonly Area[] = [
  { screen: "devices", label: "Devices", icon: "devices", key: "1" },
  { screen: "effects", label: "Effects", icon: "effects", key: "2" },
  { screen: "profiles", label: "Profiles", icon: "profiles", key: "3" },
];

export const SETTINGS_AREA: Area = { screen: "settings", label: "Settings", icon: "settings", key: "," };

export const ALL_AREAS: readonly Area[] = [...MAIN_AREAS, SETTINGS_AREA];
