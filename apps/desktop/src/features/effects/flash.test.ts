import { expect, test } from "vitest";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import { FlashLimiter, MAX_FLASHES_PER_SECOND, luminance } from "./flash";

const BLACK: Rgb = { r: 0, g: 0, b: 0 };
const WHITE: Rgb = { r: 255, g: 255, b: 255 };

/** Luminance of each drawn frame with its time, for a stream of frames offered at `hertz`. */
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
