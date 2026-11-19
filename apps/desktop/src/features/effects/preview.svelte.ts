import { api } from "../../lib/api/client";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import type { PreviewBatch } from "../../lib/api/bindings/PreviewBatch";
import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import { FlashLimiter } from "./flash";

const FRAME_INTERVAL_MS = 40;
const FRAMES_PER_BATCH = 50;
const CHANGE_DELAY_MS = 200;
const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

type Pattern = { instance: EffectInstance; leds: number[] };

function clamp(value: number, bounds: { min: number; max: number }): number {
  return Math.min(Math.max(value, bounds.min), bounds.max);
}

/** Plays preview frames rendered in Rust for each LED count on one clock; it never touches output. */
export class PreviewPlayer {
  frames = $state<Rgb[][] | null>(null);
  reduced = $state(false);
  failed = $state(false);
  slowed = $state(false);

  private pattern: Pattern | null = null;
  private key = "";
  private batches: PreviewBatch[] = [];
  private limiter = new FlashLimiter();
  private index = 0;
  private generation = 0;
  private stepTimer: ReturnType<typeof setTimeout> | undefined;
  private changeTimer: ReturnType<typeof setTimeout> | undefined;
  private motion: MediaQueryList | null = null;
  private closed = false;
  private windowShown = true;

  constructor(private readonly limits: () => PreviewLimits) {}

  open() {
    document.addEventListener("visibilitychange", this.onVisibility);
    this.motion = typeof window.matchMedia === "function" ? window.matchMedia(REDUCED_MOTION) : null;
    this.reduced = this.motion?.matches ?? false;
    this.motion?.addEventListener("change", this.onMotion);
  }

  close() {
    this.closed = true;
    this.halt();
    clearTimeout(this.changeTimer);
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.motion?.removeEventListener("change", this.onMotion);
  }

  /** The native window can be hidden while the document still reports itself visible. */
  setWindowShown(shown: boolean) {
    if (shown === this.windowShown) {
      return;
    }
    this.windowShown = shown;
    if (shown) {
      this.restart();
    } else {
      this.halt();
    }
  }

  get frame(): Rgb[] | null {
    return this.frames?.[0] ?? null;
  }

  frameFor(leds: number): Rgb[] | null {
    const frames = this.frames;
    const index = this.pattern?.leds.indexOf(clamp(leds, this.limits().leds)) ?? -1;
    return index < 0 ? null : (frames?.[index] ?? null);
  }

  show(instance: EffectInstance | null, leds: number | number[]) {
    const counts = [...new Set((Array.isArray(leds) ? leds : [leds]).map((count) => clamp(count, this.limits().leds)))];
    const pattern = instance && counts.length > 0 ? { instance, leds: counts } : null;
    const key = JSON.stringify(pattern);
    if (key === this.key) {
      return;
    }
    const first = this.key === "";
    this.key = key;
    this.pattern = pattern;
    clearTimeout(this.changeTimer);
    if (!pattern) {
      this.halt();
      this.frames = null;
      return;
    }
    this.limiter.reset();
    this.changeTimer = setTimeout(() => this.restart(), first ? 0 : CHANGE_DELAY_MS);
  }

  private onVisibility = () => {
    if (document.visibilityState === "hidden") {
      this.halt();
    } else {
      this.restart();
    }
  };

  private onMotion = (event: MediaQueryListEvent) => {
    this.reduced = event.matches;
    this.restart();
  };

  private playhead(): number {
    const batch = this.batches[0];
    return batch ? batch.start_ms + this.index * batch.interval_ms : 0;
  }

  private halt() {
    this.generation += 1;
    clearTimeout(this.stepTimer);
  }

  private restart() {
    this.halt();
    this.request(this.reduced ? 0 : this.playhead());
  }

  private async request(start: number) {
    const pattern = this.pattern;
    if (!pattern || this.closed || !this.windowShown || document.visibilityState === "hidden") {
      return;
    }
    const generation = this.generation;
    const answers = await Promise.all(
      pattern.leds.map((leds) =>
        api.previewEffect({
          instance: pattern.instance,
          led_count: leds,
          start_ms: start,
          frame_count: this.reduced ? 1 : clamp(FRAMES_PER_BATCH, this.limits().frames),
          interval_ms: clamp(FRAME_INTERVAL_MS, this.limits().interval_ms),
        }),
      ),
    );
    if (generation !== this.generation || this.closed) {
      return;
    }
    const batches = answers.flatMap((answer) => (answer.ok ? [answer.value] : []));
    this.failed = batches.length < answers.length;
    if (this.failed) {
      this.frames = null;
      return;
    }
    this.batches = batches;
    this.index = 0;
    this.step();
  }

  private step() {
    const batches = this.batches;
    const first = batches[0];
    if (!first) {
      return;
    }
    const next = batches.map((batch) => batch.frames[this.index] ?? []);
    if (this.limiter.offer(next, this.playhead())) {
      this.frames = next;
    }
    this.slowed = this.limiter.slowed;
    if (batches.every((batch) => batch.is_static) || this.reduced) {
      return;
    }
    const generation = this.generation;
    this.stepTimer = setTimeout(() => {
      if (generation !== this.generation) {
        return;
      }
      if (this.index + 1 < first.frames.length) {
        this.index += 1;
        this.step();
      } else {
        this.index = first.frames.length;
        this.request(this.playhead());
      }
    }, first.interval_ms);
  }
}
