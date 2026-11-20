import type { Screen } from "../state/app.svelte";
import type { IconName } from "./Icon.svelte";

export type Area = { screen: Screen; label: string; icon: IconName; key: string; code: string | null };

/** Keys follow the plan's order of the areas. */
export const MAIN_AREAS: readonly Area[] = [
  { screen: "devices", label: "Devices", icon: "devices", key: "1", code: "Digit1" },
  { screen: "effects", label: "Effects", icon: "effects", key: "2", code: "Digit2" },
  { screen: "setup", label: "Setup", icon: "setup", key: "3", code: "Digit3" },
  { screen: "profiles", label: "Profiles", icon: "profiles", key: "4", code: "Digit4" },
  { screen: "automation", label: "Automation", icon: "automation", key: "5", code: "Digit5" },
];

export const SETTINGS_AREA: Area = { screen: "settings", label: "Settings", icon: "settings", key: ",", code: null };

export const ALL_AREAS: readonly Area[] = [...MAIN_AREAS, SETTINGS_AREA];
