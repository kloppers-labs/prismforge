export const PANEL_MARGIN = 8;

/** How far a panel moves from its chip's left edge so that it stays inside the window. */
export function panelOffset({ triggerLeft, panelWidth, viewportWidth }: { triggerLeft: number; panelWidth: number; viewportWidth: number }): number {
  const overflow = triggerLeft + panelWidth - (viewportWidth - PANEL_MARGIN);
  const shift = overflow > 0 ? -overflow : 0;
  return Math.max(shift, PANEL_MARGIN - triggerLeft);
}
