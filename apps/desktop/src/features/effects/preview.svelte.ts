import { api } from "../../lib/api/client";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import type { PreviewBatch } from "../../lib/api/bindings/PreviewBatch";
import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
import type { Rgb } from "../../lib/api/bindings/Rgb";

const FRAME_INTERVAL_MS = 40;
const FRAMES_PER_BATCH = 50;
const CHANGE_DELAY_MS = 200;
const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

type Pattern = { instance: EffectInstance; leds: number };

function clamp(value: number, bounds: { min: number; max: number }): number {
  return Math.min(Math.max(value, bounds.min), bounds.max);
}

/** Plays preview frames rendered in Rust; it only ever asks for frames and never touches output. */
export class PreviewPlayer {
  frame = $state<Rgb[] | null>(null);
  reduced = $state(false);
  failed = $state(false);

  private pattern: Pattern | null = null;
  private key = "";
  private batch: PreviewBatch | null = null;
  private index = 0;
  private generation = 0;
  private stepTimer: ReturnType<typeof setTimeout> | undefined;
  private changeTimer: ReturnType<typeof setTimeout> | undefined;
  private motion: MediaQueryList | null = null;
  private closed = false;

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

  show(instance: EffectInstance | null, leds: number) {
    const pattern = instance ? { instance, leds: clamp(leds, this.limits().leds) } : null;
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
      this.frame = null;
      return;
    }
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
    return this.batch ? this.batch.start_ms + this.index * this.batch.interval_ms : 0;
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
    if (!pattern || this.closed || document.visibilityState === "hidden") {
      return;
    }
    const generation = this.generation;
    const answer = await api.previewEffect({
      instance: pattern.instance,
      led_count: pattern.leds,
      start_ms: start,
      frame_count: this.reduced ? 1 : clamp(FRAMES_PER_BATCH, this.limits().frames),
      interval_ms: clamp(FRAME_INTERVAL_MS, this.limits().interval_ms),
    });
    if (generation !== this.generation || this.closed) {
      return;
    }
    this.failed = !answer.ok;
    if (!answer.ok) {
      this.frame = null;
      return;
    }
    this.batch = answer.value;
    this.index = 0;
    this.step();
  }

  private step() {
    const batch = this.batch;
    if (!batch) {
      return;
    }
    this.frame = batch.frames[this.index] ?? null;
    if (batch.is_static || this.reduced) {
      return;
    }
    const generation = this.generation;
    this.stepTimer = setTimeout(() => {
      if (generation !== this.generation) {
        return;
      }
      if (this.index + 1 < batch.frames.length) {
        this.index += 1;
        this.step();
      } else {
        this.index = batch.frames.length;
        this.request(this.playhead());
      }
    }, batch.interval_ms);
  }
}
