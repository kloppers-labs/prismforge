import type { Screen } from "../state/app.svelte";
import type { IconName } from "./Icon.svelte";

export type Area = { screen: Screen; label: string; icon: IconName; key: string; code: string | null };

/** Keys follow the plan's final order; 3 (Setup) and 5 (Automation) stay free until those areas exist. */
export const MAIN_AREAS: readonly Area[] = [
  { screen: "devices", label: "Devices", icon: "devices", key: "1", code: "Digit1" },
  { screen: "effects", label: "Effects", icon: "effects", key: "2", code: "Digit2" },
  { screen: "profiles", label: "Profiles", icon: "profiles", key: "4", code: "Digit4" },
];

export const SETTINGS_AREA: Area = { screen: "settings", label: "Settings", icon: "settings", key: ",", code: null };

export const ALL_AREAS: readonly Area[] = [...MAIN_AREAS, SETTINGS_AREA];
