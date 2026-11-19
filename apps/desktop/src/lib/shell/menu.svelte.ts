import { overlays } from "../overlays.svelte";
import type { MirrorId } from "./mirrors";
import type { Box } from "./placement";

export type MenuItem = {
  id: MirrorId;
  label: string;
  run: (origin: HTMLElement) => unknown;
  reason?: string | null;
  checked?: boolean;
  danger?: boolean;
};

export type Menu = { label: string; items: MenuItem[]; focus?: "first" | "menu" };

type Opened = {
  build: () => Menu | null;
  node: HTMLElement;
  origin: HTMLElement;
  anchor: Box;
  originBox: Box;
  byKeyboard: boolean;
  at: number;
};

export const MENU_OVERLAY = "context-menu";
const POINTER_WINDOW_MS = 1000;
const MENU_SHORTCUT = "Shift+F10";
const TEXT_INPUTS = new Set(["", "text", "search", "url", "email", "password", "number", "tel"]);

class Menus {
  current = $state.raw<Opened | null>(null);

  /** `build` runs again whenever the state it reads changes, so an open menu never shows stale items. */
  open(build: () => Menu | null, node: HTMLElement, origin: HTMLElement, anchor: Box, byKeyboard: boolean) {
    overlays.popover = MENU_OVERLAY;
    this.current = { build, node, origin, anchor, originBox: boxOf(origin), byKeyboard, at: performance.now() };
  }

  close(returnFocus: boolean) {
    const opened = this.current;
    if (!opened) {
      return;
    }
    this.current = null;
    if (overlays.popover === MENU_OVERLAY) {
      overlays.popover = null;
    }
    if (returnFocus && opened.origin.isConnected) {
      opened.origin.focus();
    }
  }

  /** True right after a keyboard open, when the platform may still send its own contextmenu event for the same key. */
  echoOf(origin: HTMLElement): boolean {
    const opened = this.current;
    return opened !== null && opened.byKeyboard && opened.origin === origin && performance.now() - opened.at < POINTER_WINDOW_MS;
  }
}

export const menus = new Menus();

/** Normal items first, destructive items last, so a separator can sit between the two groups. */
export function groups(items: MenuItem[]): MenuItem[][] {
  return [items.filter((item) => !item.danger), items.filter((item) => item.danger)].filter((group) => group.length > 0);
}

export function isTextField(target: EventTarget | null): boolean {
  if (target instanceof HTMLTextAreaElement) {
    return true;
  }
  if (target instanceof HTMLInputElement) {
    return TEXT_INPUTS.has(target.getAttribute("type")?.toLowerCase() ?? "");
  }
  return target instanceof HTMLElement && (target.isContentEditable || target.closest("[contenteditable]:not([contenteditable='false'])") !== null);
}

export function isMenuKey(event: KeyboardEvent): boolean {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return false;
  }
  return event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey);
}

function insideInert(element: HTMLElement): boolean {
  for (let node: HTMLElement | null = element; node; node = node.parentElement) {
    if (node.inert || node.hasAttribute("inert")) {
      return true;
    }
  }
  return false;
}

function canOpen(origin: HTMLElement): boolean {
  if (insideInert(origin)) {
    return false;
  }
  const modal = document.querySelector("[aria-modal='true']");
  return modal === null || modal.contains(origin);
}

export function boxOf(element: HTMLElement): Box {
  const { left, top, right, bottom } = element.getBoundingClientRect();
  return { left, top, right, bottom };
}

function overlap(a: Box, b: Box): boolean {
  return a.bottom > b.top && a.top < b.bottom && a.right > b.left && a.left < b.right;
}

/** False once the item has scrolled out of the window or out of a pane that clips it; unmeasured boxes count as shown. */
export function inView(element: HTMLElement): boolean {
  const own = element.getBoundingClientRect();
  if (own.width === 0 && own.height === 0) {
    return true;
  }
  if (!overlap(own, { left: 0, top: 0, right: window.innerWidth, bottom: window.innerHeight })) {
    return false;
  }
  for (let node = element.parentElement; node; node = node.parentElement) {
    const style = getComputedStyle(node);
    const clip = node.getBoundingClientRect();
    if (/auto|scroll|hidden/.test(style.overflowX + style.overflowY) && clip.width > 0 && clip.height > 0 && !overlap(own, clip)) {
      return false;
    }
  }
  return true;
}

const FOCUSABLE = "button:not(:disabled), input:not(:disabled), summary, [href], [tabindex]";

function originFor(node: HTMLElement, target: EventTarget | null): HTMLElement {
  if (node.matches(FOCUSABLE)) {
    return node;
  }
  const inside = target instanceof Element ? target.closest<HTMLElement>(FOCUSABLE) : null;
  if (inside && node.contains(inside)) {
    return inside;
  }
  return node.querySelector<HTMLElement>(FOCUSABLE) ?? node;
}

let pointerAt = Number.NEGATIVE_INFINITY;
let watching = false;

function watchPointer() {
  if (!watching) {
    watching = true;
    window.addEventListener("pointerdown", () => (pointerAt = performance.now()), { capture: true });
  }
}

/** Gives `node` a context menu, opened by a secondary click, Shift+F10 or the Menu key. */
export function contextMenu(node: HTMLElement, build: (() => Menu | null) | null | undefined) {
  let current = build;
  watchPointer();

  function show(event: Event, origin: HTMLElement, anchor: Box, byKeyboard: boolean) {
    const menu = current?.() ?? null;
    if (!menu || menu.items.length === 0 || !canOpen(origin)) {
      return;
    }
    event.preventDefault();
    menus.open(() => current?.() ?? null, node, origin, anchor, byKeyboard);
  }

  function oncontextmenu(event: MouseEvent) {
    if (event.defaultPrevented || isTextField(event.target)) {
      return;
    }
    const origin = originFor(node, event.target);
    if (menus.echoOf(origin)) {
      event.preventDefault();
      return;
    }
    const pointer = performance.now() - pointerAt < POINTER_WINDOW_MS;
    const anchor = pointer ? { left: event.clientX, top: event.clientY, right: event.clientX, bottom: event.clientY } : boxOf(origin);
    show(event, origin, anchor, !pointer);
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.defaultPrevented || !isMenuKey(event) || isTextField(event.target)) {
      return;
    }
    const active = document.activeElement;
    const origin = active instanceof HTMLElement && node.contains(active) ? active : node;
    show(event, origin, boxOf(origin), true);
  }

  const announced = node.matches(FOCUSABLE) ? node.getAttribute("aria-keyshortcuts") : undefined;
  if (announced !== undefined) {
    node.setAttribute("aria-keyshortcuts", announced ? `${announced} ${MENU_SHORTCUT}` : MENU_SHORTCUT);
  }
  node.addEventListener("contextmenu", oncontextmenu);
  node.addEventListener("keydown", onkeydown);
  return {
    update(next: (() => Menu | null) | null | undefined) {
      current = next;
    },
    destroy() {
      if (menus.current?.node === node) {
        menus.close(false);
      }
      node.removeEventListener("contextmenu", oncontextmenu);
      node.removeEventListener("keydown", onkeydown);
      if (announced) {
        node.setAttribute("aria-keyshortcuts", announced);
      } else if (announced === null) {
        node.removeAttribute("aria-keyshortcuts");
      }
    },
  };
}
