import { isMac } from "../platform";
import { ALL_AREAS, type Area } from "./areas";

export function shortcutKeys(area: Area): string {
  return `${isMac() ? "Meta" : "Control"}+${area.key}`;
}

export function shortcutText(area: Area): string {
  return isMac() ? `Command-${area.key}` : `Ctrl+${area.key}`;
}

export function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  return target.isContentEditable || target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement;
}

function withModifier(event: KeyboardEvent): boolean {
  const primary = isMac() ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  return primary && !event.altKey && !event.shiftKey;
}

export function areaFor(event: KeyboardEvent): Area | null {
  if (event.repeat || !withModifier(event) || isTyping(event.target) || document.querySelector("[aria-modal='true']")) {
    return null;
  }
  return ALL_AREAS.find((area) => area.code !== null && area.code === event.code) ?? ALL_AREAS.find((area) => area.key === event.key) ?? null;
}

export function isQuit(event: KeyboardEvent): boolean {
  return !event.repeat && !isMac() && withModifier(event) && event.key.toLowerCase() === "q";
}

export function cycleRegion(regions: (HTMLElement | null | undefined)[], backwards: boolean) {
  const present = regions.filter((region): region is HTMLElement => !!region);
  const active = document.activeElement;
  const at = present.findIndex((region) => region === active || region.contains(active));
  const step = backwards ? -1 : 1;
  const start = at === -1 ? (backwards ? present.length : -1) : at;
  for (let tried = 1; tried <= present.length; tried += 1) {
    const region = present[(start + step * tried + present.length * 2) % present.length]!;
    focusRegion(region);
    if (region.contains(document.activeElement)) {
      return;
    }
  }
}

function focusRegion(region: HTMLElement) {
  const marked = region.querySelector<HTMLElement>("[aria-current='page'], [data-region-focus]");
  const own = region.getAttribute("tabindex") === "-1" ? region : null;
  const first = region.querySelector<HTMLElement>("button:not(:disabled), a[href], input:not(:disabled)");
  (own ?? marked ?? first ?? region).focus();
}
