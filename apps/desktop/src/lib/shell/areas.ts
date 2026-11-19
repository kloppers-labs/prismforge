import type { Screen } from "../state/app.svelte";
import type { IconName } from "./Icon.svelte";

export type Area = { screen: Screen; label: string; icon: IconName; key: string };

const MAIN: readonly Omit<Area, "key">[] = [
  { screen: "devices", label: "Devices", icon: "devices" },
  { screen: "effects", label: "Effects", icon: "effects" },
  { screen: "profiles", label: "Profiles", icon: "profiles" },
];

export const MAIN_AREAS: readonly Area[] = MAIN.map((area, index) => ({ ...area, key: String(index + 1) }));

export const SETTINGS_AREA: Area = { screen: "settings", label: "Settings", icon: "settings", key: "," };

export const ALL_AREAS: readonly Area[] = [...MAIN_AREAS, SETTINGS_AREA];
