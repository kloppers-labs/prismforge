import type { UnlistenFn } from "@tauri-apps/api/event";
import { api } from "../../lib/api/client";
import type { LiveOutput } from "../../lib/api/bindings/LiveOutput";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import { FlashLimiter } from "./flash";

const STILL_EVERY_MS = 2000;
const ASK_AGAIN_MS = 1000;

/** Shows the frames PrismForge sends; it asks for them only while the view is on screen and never writes. */
export class LiveWatcher {
  output = $state<LiveOutput | null>(null);
  slowed = $state(false);
  reduced = $state(false);

  private limiter = new FlashLimiter();
  private asked = "[]";
  private unlisten: UnlistenFn | null = null;
  private lastDrawn = Number.NEGATIVE_INFINITY;
  private closed = false;
  private again: ReturnType<typeof setTimeout> | undefined;

  async open() {
    const unlisten = await api.onLive((live) => this.take(live));
    if (this.closed) {
      unlisten();
    } else {
      this.unlisten = unlisten;
    }
  }

  close() {
    this.closed = true;
    clearTimeout(this.again);
    this.unlisten?.();
    this.watch([]);
  }

  /** An empty list, or a hidden view, stops the feed in Rust. */
  watch(targets: OutputTarget[]) {
    const asked = JSON.stringify(targets);
    if (asked === this.asked) {
      return;
    }
    clearTimeout(this.again);
    this.asked = asked;
    this.limiter.reset();
    this.lastDrawn = Number.NEGATIVE_INFINITY;
    if (targets.length === 0) {
      this.output = null;
    }
    void this.ask(targets, asked);
  }

  /** A refused request is asked again, so the view never shows a stopped feed as live. */
  private async ask(targets: OutputTarget[], asked: string) {
    const answer = await api.watchLiveOutput(targets);
    if (answer.ok || asked !== this.asked || this.closed) {
      return;
    }
    this.asked = "";
    this.again = setTimeout(() => this.watch(targets), ASK_AGAIN_MS);
  }

  private take(live: LiveOutput) {
    if (this.asked === "[]") {
      return;
    }
    const now = performance.now();
    if (this.reduced && now - this.lastDrawn < STILL_EVERY_MS) {
      return;
    }
    const rows = live.devices.flatMap((device) => device.zones.map((zone) => zone.colors));
    if (this.limiter.offer(rows, now)) {
      this.output = live;
      this.lastDrawn = now;
    }
    this.slowed = this.limiter.slowed;
  }
}
