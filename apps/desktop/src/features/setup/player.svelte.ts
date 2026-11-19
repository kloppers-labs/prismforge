import { api } from "../../lib/api/client";
import type { AcrossPreview } from "../../lib/api/bindings/AcrossPreview";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import type { Flow } from "../../lib/api/bindings/Flow";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
import type { Rgb } from "../../lib/api/bindings/Rgb";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { failureText } from "../../lib/errors";
import { FlashLimiter } from "../effects/flash";
import { sameTarget } from "./wiring";

const FRAME_INTERVAL_MS = 40;
const FRAMES_PER_BATCH = 50;
const CHANGE_DELAY_MS = 200;
const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

export type Draft = { setup: Setup; view: ViewId; flow: Flow; instance: EffectInstance };

function clamp(value: number, bounds: { min: number; max: number }): number {
  return Math.min(Math.max(value, bounds.min), bounds.max);
}

/** Plays a draft of an effect across a view, rendered in Rust from the unsaved drawing; it never touches output. */
export class AcrossPlayer {
  frame = $state<{ target: OutputTarget; colors: Rgb[] }[] | null>(null);
  reduced = $state(false);
  problem = $state<string | null>(null);
  slowed = $state(false);

  private draft: Draft | null = null;
  private key = "";
  private batch: AcrossPreview | null = null;
  private limiter = new FlashLimiter();
  private index = 0;
  private generation = 0;
  private stepTimer: ReturnType<typeof setTimeout> | undefined;
  private changeTimer: ReturnType<typeof setTimeout> | undefined;
  private motion: MediaQueryList | null = null;
  private closed = false;
  private windowShown = true;

  constructor(private readonly limits: () => PreviewLimits | null) {}

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

  colorsOf(target: OutputTarget): Rgb[] | null {
    return this.frame?.find((each) => sameTarget(each.target, target))?.colors ?? null;
  }

  show(draft: Draft | null) {
    const key = JSON.stringify(draft);
    if (key === this.key) {
      return;
    }
    const first = this.key === "";
    this.key = key;
    this.draft = draft;
    clearTimeout(this.changeTimer);
    if (!draft) {
      this.halt();
      this.frame = null;
      this.problem = null;
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
    return this.batch ? this.batch.start_ms + this.index * this.batch.interval_ms : 0;
  }

  private halt() {
    this.generation += 1;
    clearTimeout(this.stepTimer);
  }

  private restart() {
    this.halt();
    void this.request(this.reduced ? 0 : this.playhead());
  }

  private async request(start: number) {
    const draft = this.draft;
    const limits = this.limits();
    if (!draft || !limits || this.closed || !this.windowShown || document.visibilityState === "hidden") {
      return;
    }
    const generation = this.generation;
    const answer = await api.previewAcross({
      ...draft,
      start_ms: start,
      frame_count: this.reduced ? 1 : clamp(FRAMES_PER_BATCH, limits.frames),
      interval_ms: clamp(FRAME_INTERVAL_MS, limits.interval_ms),
    });
    if (generation !== this.generation || this.closed) {
      return;
    }
    if (!answer.ok) {
      this.problem = failureText(answer.error);
      this.frame = null;
      return;
    }
    this.problem = null;
    this.batch = answer.value;
    this.index = 0;
    this.step();
  }

  private step() {
    const batch = this.batch;
    if (!batch) {
      return;
    }
    const next = batch.targets.map((target) => ({ target: target.target, colors: target.frames[this.index] ?? [] }));
    if (this.limiter.offer(next.map((each) => each.colors), this.playhead())) {
      this.frame = next;
    }
    this.slowed = this.limiter.slowed;
    const length = batch.targets[0]?.frames.length ?? 0;
    if (batch.is_static || this.reduced || length === 0) {
      return;
    }
    const generation = this.generation;
    this.stepTimer = setTimeout(() => {
      if (generation !== this.generation) {
        return;
      }
      if (this.index + 1 < length) {
        this.index += 1;
        this.step();
      } else {
        this.index = length;
        void this.request(this.playhead());
      }
    }, batch.interval_ms);
  }
}
