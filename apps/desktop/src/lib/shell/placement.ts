export const PANEL_MARGIN = 8;

/** How far a panel moves from its chip's left edge so that it stays inside the window. */
export function panelOffset({ triggerLeft, panelWidth, viewportWidth }: { triggerLeft: number; panelWidth: number; viewportWidth: number }): number {
  const overflow = triggerLeft + panelWidth - (viewportWidth - PANEL_MARGIN);
  const shift = overflow > 0 ? -overflow : 0;
  return Math.max(shift, PANEL_MARGIN - triggerLeft);
}

export type Box = { left: number; top: number; right: number; bottom: number };

type Size = { width: number; height: number };

function clamp(value: number, size: number, room: number): number {
  return Math.max(PANEL_MARGIN, Math.min(value, room - PANEL_MARGIN - size));
}

function along(start: number, end: number, size: number, room: number): number {
  if (start + size <= room - PANEL_MARGIN) {
    return start;
  }
  if (end - size >= PANEL_MARGIN) {
    return end - size;
  }
  return clamp(start, size, room);
}

/** Where a context menu goes: after its anchor when it fits, else before it, else clamped inside the window. */
export function menuPlacement(anchor: Box, size: Size, viewport: Size): { left: number; top: number } {
  const left = along(anchor.left, anchor.right, size.width, viewport.width);
  const top = along(anchor.bottom, anchor.top, size.height, viewport.height);
  return { left: clamp(left, size.width, viewport.width), top: clamp(top, size.height, viewport.height) };
}
