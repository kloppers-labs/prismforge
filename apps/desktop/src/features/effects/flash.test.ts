import { expect, test } from "vitest";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import { FlashLimiter, MAX_FLASHES_PER_SECOND, luminance } from "./flash";

const BLACK: Rgb = { r: 0, g: 0, b: 0 };
const WHITE: Rgb = { r: 255, g: 255, b: 255 };

function play(frames: (index: number) => Rgb[][], hertz: number, seconds: number, limiter = new FlashLimiter()) {
  const drawn: { at: number; light: number }[] = [];
  const period = 1000 / hertz;
  for (let index = 0; index < hertz * seconds; index += 1) {
    const at = index * period;
    const frame = frames(index);
    if (limiter.offer(frame, at)) {
      drawn.push({ at, light: luminance(frame[0] ?? []) });
    }
  }
  return { drawn, limiter };
}

/** The most flashes (pairs of opposing changes of 10% or more) in any one second of what was drawn. */
function worstSecond(drawn: { at: number; light: number }[]): number {
  const changes: number[] = [];
  for (let index = 1; index < drawn.length; index += 1) {
    const before = drawn[index - 1]!.light;
    const after = drawn[index]!.light;
    if (Math.abs(after - before) >= 0.1 && Math.min(after, before) < 0.8) {
      changes.push(drawn[index]!.at);
    }
  }
  let worst = 0;
  for (const start of changes) {
    worst = Math.max(worst, changes.filter((at) => at >= start && at < start + 1000).length);
  }
  return Math.floor(worst / 2);
}

test("a_synthetic_strobe_is_drawn_with_at_most_three_flashes_each_second", () => {
  const strobe = (index: number) => [Array(12).fill(index % 2 === 0 ? BLACK : WHITE)];

  const { drawn, limiter } = play(strobe, 30, 4);

  expect(MAX_FLASHES_PER_SECOND).toBe(3);
  expect(worstSecond(drawn)).toBeLessThanOrEqual(3);
  expect(drawn.length).toBeGreaterThan(4);
  expect(limiter.slowed).toBe(true);
});

test("a_strobe_on_one_row_of_several_is_limited_too", () => {
  const calm = Array(8).fill({ r: 40, g: 40, b: 40 });
  const strobe = (index: number) => [calm, Array(30).fill(index % 3 === 0 ? WHITE : BLACK)];

  const limiter = new FlashLimiter();
  const drawn: { at: number; light: number }[] = [];
  for (let index = 0; index < 120; index += 1) {
    const frame = strobe(index);
    if (limiter.offer(frame, index * 25)) {
      drawn.push({ at: index * 25, light: luminance(frame[1]!) });
    }
  }

  expect(worstSecond(drawn)).toBeLessThanOrEqual(3);
});

test("a_slow_change_is_drawn_frame_by_frame_and_not_marked_as_slowed", () => {
  const fade = (index: number) => {
    const level = Math.round(128 + 120 * Math.sin(index / 40));
    return [Array(12).fill({ r: level, g: level, b: level })];
  };

  const { drawn, limiter } = play(fade, 25, 4);

  expect(drawn).toHaveLength(100);
  expect(limiter.slowed).toBe(false);
});

test("the_relative_luminance_follows_the_srgb_formula", () => {
  expect(luminance([WHITE])).toBeCloseTo(1, 5);
  expect(luminance([BLACK])).toBe(0);
  expect(luminance([{ r: 255, g: 0, b: 0 }])).toBeCloseTo(0.2126, 4);
  expect(luminance([])).toBe(0);
});

function hsv(hue: number): Rgb {
  const h = ((hue % 1) + 1) % 1;
  const sector = Math.floor(h * 6);
  const f = h * 6 - sector;
  const up = Math.round(255 * f);
  const down = Math.round(255 * (1 - f));
  return [
    { r: 255, g: up, b: 0 },
    { r: down, g: 255, b: 0 },
    { r: 0, g: 255, b: up },
    { r: 0, g: down, b: 255 },
    { r: up, g: 0, b: 255 },
    { r: 255, g: 0, b: down },
  ][sector]!;
}

/** The most opposing swings of 0.1 or more that any LED makes in one second of what was drawn, as flashes. */
function worstLedSecond(drawn: { at: number; leds: number[] }[]): number {
  let worst = 0;
  const count = drawn[0]?.leds.length ?? 0;
  for (let led = 0; led < count; led += 1) {
    const swings: number[] = [];
    let peak = drawn[0]!.leds[led]!;
    let direction = 0;
    for (const frame of drawn.slice(1)) {
      const value = frame.leds[led]!;
      const turn = direction >= 0 ? peak - value : value - peak;
      if ((direction >= 0 && value > peak) || (direction < 0 && value < peak)) {
        peak = value;
      } else if (turn >= 0.1 && Math.min(value, peak) < 0.8) {
        swings.push(frame.at);
        direction = direction >= 0 ? -1 : 1;
        peak = value;
      }
    }
    for (const start of swings) {
      worst = Math.max(worst, swings.filter((at) => at >= start && at < start + 1000).length);
    }
  }
  return Math.floor(worst / 2);
}

function playLeds(frames: (index: number) => Rgb[][], hertz: number, seconds: number) {
  const limiter = new FlashLimiter();
  const drawn: { at: number; leds: number[] }[] = [];
  for (let index = 0; index < hertz * seconds; index += 1) {
    const at = (index * 1000) / hertz;
    const frame = frames(index);
    if (limiter.offer(frame, at)) {
      drawn.push({ at, leds: frame.flat().map((led) => luminance([led])) });
    }
  }
  return { drawn, limiter };
}

test("a_rainbow_wave_at_top_speed_is_slowed_although_each_row_keeps_its_mean", () => {
  const cyclesPerSecond = 240 / 60;
  const wave = (index: number) => [Array.from({ length: 30 }, (_, led) => hsv(led / 30 + (index / 25) * cyclesPerSecond))];

  const { drawn, limiter } = playLeds(wave, 25, 4);

  expect(worstLedSecond(drawn)).toBeLessThanOrEqual(3);
  expect(limiter.slowed).toBe(true);
});

test("a_half_on_half_off_alternation_is_slowed", () => {
  const halves = (index: number) => [Array.from({ length: 20 }, (_, led) => ((led < 10) === (index % 2 === 0) ? WHITE : BLACK))];

  const { drawn, limiter } = playLeds(halves, 30, 3);

  expect(worstLedSecond(drawn)).toBeLessThanOrEqual(3);
  expect(limiter.slowed).toBe(true);
});
