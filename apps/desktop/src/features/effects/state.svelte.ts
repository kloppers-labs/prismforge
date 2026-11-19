import { api, type Answer, type Failure } from "../../lib/api/client";
import type { DeviceRef } from "../../lib/api/bindings/DeviceRef";
import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { ShellView } from "../../lib/api/bindings/ShellView";
import type { Names } from "../../lib/names";
import { serverName } from "../connection/wording";
import {
  currentEffect,
  currentFrameRate,
  currentMode,
  type EffectFeedback,
  type FrameRateFeedback,
  type ModeFeedback,
} from "./feedback";
import { applyBlocked, applyingText, deviceOf, runningOn, targetName } from "./present";
import { check, initialFields, parseHex, problemText, START_COLOR, toHex, type Field, type Fields } from "./values";

const REAPPLY_DELAY_MS = 300;

function deviceHold(device: DeviceRef): string {
  return `device|${device}`;
}

function targetHold(target: OutputTarget): string {
  return `target|${target.device}|${target.zone ?? "all"}`;
}

export class EffectsState {
  catalog = $state<EffectCatalog | null>(null);
  catalogProblem = $state<Failure | null>(null);
  target = $state<OutputTarget | null>(null);
  effect = $state<string | null>(null);
  fields = $state<Fields>({});
  rustProblems = $state<Record<string, string>>({});
  working = $state<"apply" | "stop" | null>(null);
  private lastColor = START_COLOR;
  externalProblem = $state<Failure | null>(null);
  private effectResult = $state<EffectFeedback | null>(null);
  private modeResult = $state<ModeFeedback | null>(null);
  private frameRateResult = $state<FrameRateFeedback | null>(null);
  /** Devices and targets where the user's last explicit action forbids automatic re-apply until Apply. */
  private held = $state<string[]>([]);
  private inFlight = $state<Record<string, number>>({});

  definition = $derived(this.catalog?.effects.find((effect) => effect.id === this.effect) ?? null);
  checked = $derived(this.definition ? check(this.definition, this.fields) : null);
  feedback = $derived.by(() => currentEffect(this.effectResult, this.view()));
  modeFeedback = $derived.by(() => currentMode(this.modeResult, this.view()));
  frameRateFeedback = $derived.by(() => currentFrameRate(this.frameRateResult, this.view()));
  running = $derived.by(() => {
    const session = this.view().session;
    return session && this.target ? runningOn(session, this.target) : null;
  });
  blocked = $derived.by(() => {
    const session = this.view().session;
    const device = session && this.target ? deviceOf(session, this.target) : null;
    if (!session || !this.target || !device) {
      return "This device is not in the device list.";
    }
    return applyingText(this.view()) ?? applyBlocked(session, device, this.target.zone);
  });
  /** True only while a changed setting may be sent without the user pressing Apply. */
  canAutoApply = $derived(
    this.target !== null &&
      this.running !== null &&
      this.running.effect.effect === this.effect &&
      this.blocked === null &&
      !this.isHeld(this.target),
  );

  private reapplyTimer: ReturnType<typeof setTimeout> | undefined;
  private reapplyAfterCall = false;

  constructor(
    private readonly view: () => ShellView,
    private readonly names: () => Names,
    private readonly changed: () => void,
  ) {}

  private loading: Promise<void> | null = null;

  load(): Promise<void> {
    if (this.catalog) {
      return Promise.resolve();
    }
    this.loading ??= this.fetchCatalog().finally(() => (this.loading = null));
    return this.loading;
  }

  private async fetchCatalog() {
    this.catalogProblem = null;
    const answer = await api.effectCatalog();
    if (answer.ok) {
      this.catalog = answer.value;
    } else {
      this.catalogProblem = answer.error;
    }
  }

  reset() {
    this.cancelReapply();
    this.held = [];
    this.target = null;
    this.effect = null;
    this.fields = {};
    this.clearResults();
  }

  /** The user left the screen or the session; a pending change is dropped and the target waits for Apply. */
  leave() {
    this.cancelReapply();
    if (this.target) {
      this.hold(targetHold(this.target));
    }
  }

  select(target: OutputTarget) {
    this.leave();
    this.target = target;
    this.effect = null;
    this.fields = {};
    this.clearResults();
    const running = this.running;
    if (running) {
      this.effect = running.effect.effect;
      this.fields = this.definition ? initialFields(this.definition, running.effect.values) : {};
    }
  }

  choose(effect: string) {
    this.cancelReapply();
    this.effect = effect;
    this.rustProblems = {};
    const running = this.running;
    const values = running?.effect.effect === effect ? running.effect.values : {};
    this.fields = this.definition ? initialFields(this.definition, values, this.lastColor) : {};
  }

  setField(id: string, field: Field) {
    this.fields = { ...this.fields, [id]: field };
    const color = field.kind === "color" ? parseHex(field.text) : null;
    if (color) {
      this.lastColor = toHex(color);
    }
    const { [id]: _, ...others } = this.rustProblems;
    this.rustProblems = others;
    if (this.canAutoApply) {
      this.scheduleReapply();
    }
  }

  async apply() {
    this.cancelReapply();
    const target = this.target;
    if (target) {
      this.held = this.held.filter((key) => key !== deviceHold(target.device) && key !== targetHold(target));
    }
    await this.send();
  }

  async stop() {
    const target = this.target;
    if (!target) {
      return;
    }
    this.holdDevice(target.device);
    this.working = "stop";
    const answer = await this.call(target.device, () => api.stopEffect(target));
    this.working = null;
    if (answer.ok) {
      this.changed();
      this.modeResult = null;
      this.effectResult = { kind: "stopped", outcome: answer.value, target, revision: this.view().revision };
    } else {
      this.effectResult = { kind: "failed", error: answer.error };
    }
  }

  async selectMode(device: DeviceRef, mode: number, name: string) {
    this.holdDevice(device);
    const answer = await this.call(device, () => api.selectMode(device, mode));
    if (answer.ok) {
      this.changed();
      this.effectResult = null;
      const session = this.view().session;
      this.modeResult = {
        kind: "selected",
        device,
        mode,
        name,
        server: session ? serverName(session) : "OpenRGB",
        revision: this.view().revision,
      };
    } else {
      this.modeResult = { kind: "failed", error: answer.error };
    }
  }

  async declareExternal(device: DeviceRef, label: string) {
    this.holdDevice(device);
    const answer = await this.call(device, () => api.declareExternal(device, label));
    this.afterOwnership(answer);
  }

  async clearExternal(device: DeviceRef) {
    this.holdDevice(device);
    const answer = await this.call(device, () => api.clearExternal(device));
    this.afterOwnership(answer);
  }

  async setFrameRate(requested: number) {
    const answer = await api.setFrameRate(requested);
    if (answer.ok) {
      this.changed();
    }
    this.frameRateResult = answer.ok
      ? { kind: "set", requested, used: answer.value, revision: this.view().revision }
      : { kind: "failed", error: answer.error };
  }

  private async send() {
    const target = this.target;
    const instance = this.checked?.instance;
    const definition = this.definition;
    if (!target || !instance || !definition) {
      return;
    }
    this.working = "apply";
    const answer = await this.call(target.device, () => api.applyEffect(target, instance));
    this.working = null;
    if (answer.ok) {
      this.changed();
      this.rustProblems = {};
      this.modeResult = null;
      this.effectResult = {
        kind: "applied",
        effect: definition.id,
        label: definition.label,
        target,
        targetLabel: this.targetLabel(target),
        revision: this.view().revision,
      };
      return;
    }
    const error = answer.error;
    if (typeof error === "object" && "InvalidEffect" in error && "InvalidValues" in error.InvalidEffect) {
      this.rustProblems = Object.fromEntries(
        error.InvalidEffect.InvalidValues.errors.map((value) => [value.control, problemText(value.problem)]),
      );
    }
    this.effectResult = { kind: "failed", error };
  }

  private autoApply() {
    this.reapplyTimer = undefined;
    if (!this.canAutoApply || !this.target) {
      return;
    }
    if ((this.inFlight[this.target.device] ?? 0) > 0) {
      this.reapplyAfterCall = true;
      return;
    }
    this.send();
  }

  private async call<T>(device: DeviceRef, command: () => Promise<Answer<T>>): Promise<Answer<T>> {
    this.inFlight = { ...this.inFlight, [device]: (this.inFlight[device] ?? 0) + 1 };
    try {
      return await command();
    } finally {
      this.inFlight = { ...this.inFlight, [device]: (this.inFlight[device] ?? 1) - 1 };
      if (this.reapplyAfterCall && this.inFlight[device] === 0) {
        this.reapplyAfterCall = false;
        if (this.canAutoApply) {
          this.scheduleReapply();
        }
      }
    }
  }

  private afterOwnership(answer: Answer<null>) {
    if (answer.ok) {
      this.changed();
      this.effectResult = null;
      this.modeResult = null;
      this.externalProblem = null;
    } else {
      this.externalProblem = answer.error;
    }
  }

  private isHeld(target: OutputTarget): boolean {
    return this.held.includes(deviceHold(target.device)) || this.held.includes(targetHold(target));
  }

  private holdDevice(device: DeviceRef) {
    this.cancelReapply();
    this.hold(deviceHold(device));
  }

  private hold(key: string) {
    if (!this.held.includes(key)) {
      this.held = [...this.held, key];
    }
  }

  private targetLabel(target: OutputTarget): string {
    const session = this.view().session;
    const device = session ? deviceOf(session, target) : null;
    return device ? targetName(this.names(), device, target.zone) : target.device;
  }

  private clearResults() {
    this.rustProblems = {};
    this.effectResult = null;
    this.modeResult = null;
    this.externalProblem = null;
  }

  private scheduleReapply() {
    this.cancelReapply();
    this.reapplyTimer = setTimeout(() => this.autoApply(), REAPPLY_DELAY_MS);
  }

  private cancelReapply() {
    clearTimeout(this.reapplyTimer);
    this.reapplyTimer = undefined;
    this.reapplyAfterCall = false;
  }
}
