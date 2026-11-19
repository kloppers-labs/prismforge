import type { Rgb } from "../../lib/api/bindings/Rgb";

/** WCAG 2.3.1; it limits the views on the screen only, never the frames sent to the devices. */
export const MAX_FLASHES_PER_SECOND = 3;

const WINDOW_MS = 1000;
const MAX_CHANGES = MAX_FLASHES_PER_SECOND * 2;
const SIGNIFICANT = 0.1;
const DARKER_BELOW = 0.8;
const SLOWED_FOR_MS = 2000;

function linear(channel: number): number {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

/** The mean relative luminance of a row of LEDs, from 0 to 1. */
export function luminance(leds: Rgb[]): number {
  if (leds.length === 0) {
    return 0;
  }
  const sum = leds.reduce((total, led) => total + 0.2126 * linear(led.r) + 0.7152 * linear(led.g) + 0.0722 * linear(led.b), 0);
  return sum / leds.length;
}

/**
 * Decides which frames a view draws: a frame that would make a fourth flash within one second is held back.
 * A flash is a pair of opposing changes of 10% or more in any row's luminance, with the darker side below 0.8.
 */
export class FlashLimiter {
  slowed = false;
  private shown: number[] | null = null;
  private changes: number[] = [];
  private lastHeld = Number.NEGATIVE_INFINITY;

  offer(rows: Rgb[][], now: number): boolean {
    const light = rows.map(luminance);
    const before = this.shown;
    const significant =
      before !== null &&
      light.some((value, index) => {
        const was = before[index] ?? value;
        return Math.abs(value - was) >= SIGNIFICANT && Math.min(value, was) < DARKER_BELOW;
      });
    this.changes = this.changes.filter((at) => at > now - WINDOW_MS);
    if (significant && this.changes.length >= MAX_CHANGES) {
      this.lastHeld = now;
      this.slowed = true;
      return false;
    }
    if (significant) {
      this.changes.push(now);
    }
    this.shown = light;
    if (now - this.lastHeld > SLOWED_FOR_MS) {
      this.slowed = false;
    }
    return true;
  }

  reset() {
    this.shown = null;
    this.changes = [];
  }
}
