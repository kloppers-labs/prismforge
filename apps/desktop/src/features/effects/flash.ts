import type { Rgb } from "../../lib/api/bindings/Rgb";

/** WCAG 2.3.1; it limits the views on the screen only, never the frames sent to the devices. */
export const MAX_FLASHES_PER_SECOND = 3;

const WINDOW_MS = 1000;
const MAX_CHANGES = MAX_FLASHES_PER_SECOND * 2;
const SIGNIFICANT = 0.1;
const DARKER_BELOW = 0.8;
const SLOWED_FOR_MS = 2000;
const AREA = 0.1;

function linear(channel: number): number {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

export function luminance(leds: Rgb[]): number {
  if (leds.length === 0) {
    return 0;
  }
  const sum = leds.reduce((total, led) => total + 0.2126 * linear(led.r) + 0.7152 * linear(led.g) + 0.0722 * linear(led.b), 0);
  return sum / leds.length;
}

/** One LED's swing: the extreme it reached since its last counted change, and which way it went. */
type Swing = { peak: number; direction: number; changes: number[] };

/**
 * Holds back a frame that would give a tenth or more of the LEDs a fourth flash within one second.
 * A flash is a pair of opposing swings of 10% or more in an LED's relative luminance, with the darker side below 0.8.
 */
export class FlashLimiter {
  slowed = false;
  private swings: Swing[] | null = null;
  private lastHeld = Number.NEGATIVE_INFINITY;

  offer(rows: Rgb[][], now: number): boolean {
    const light = rows.flat().map((led) => luminance([led]));
    if (this.swings === null || this.swings.length !== light.length) {
      this.swings = light.map((value) => ({ peak: value, direction: 0, changes: [] }));
      return this.drawn(now);
    }
    const swings = this.swings;
    const turns = light.map((value, index) => turn(swings[index]!, value));
    const over = turns.filter(
      (turned, index) => turned !== 0 && swings[index]!.changes.filter((at) => at > now - WINDOW_MS).length >= MAX_CHANGES,
    ).length;
    if (over >= Math.max(1, light.length * AREA)) {
      this.lastHeld = now;
      this.slowed = true;
      return false;
    }
    light.forEach((value, index) => {
      const swing = swings[index]!;
      const turned = turns[index]!;
      if (turned !== 0) {
        swing.direction = turned;
        swing.peak = value;
        swing.changes = [...swing.changes.filter((at) => at > now - WINDOW_MS), now];
      } else if (swing.direction !== 0 && (value - swing.peak) * swing.direction > 0) {
        swing.peak = value;
      }
    });
    return this.drawn(now);
  }

  reset() {
    this.swings = null;
  }

  private drawn(now: number): boolean {
    if (now - this.lastHeld > SLOWED_FOR_MS) {
      this.slowed = false;
    }
    return true;
  }
}

/** The direction of a new counted swing, or 0 when the LED keeps its way or moved too little. */
function turn(swing: Swing, value: number): number {
  const moved = value - swing.peak;
  const direction = Math.sign(moved);
  if (direction === 0 || direction === swing.direction) {
    return 0;
  }
  return Math.abs(moved) >= SIGNIFICANT && Math.min(value, swing.peak) < DARKER_BELOW ? direction : 0;
}
