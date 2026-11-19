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
import type { BatchReport } from "../../lib/api/bindings/BatchReport";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import { isLive } from "../devices/present";
import { applyBlocked, applyingText, deviceOf, runningOn, sameTarget, targetName } from "./present";
import { retryable } from "./retry";
import { addRange, effective, selectAll, toggle } from "./selection";
import { check, initialFields, parseHex, problemText, START_COLOR, toHex, type Field, type Fields } from "./values";

const REAPPLY_DELAY_MS = 300;

function deviceHold(device: DeviceRef): string {
  return `device|${device}`;
}

function targetHold(target: OutputTarget): string {
  return `target|${target.device}|${target.zone ?? "all"}`;
}

export function targetsKey(targets: OutputTarget[]): string {
  return targets.map((target) => `${target.device}|${target.zone ?? "all"}`).join(",");
}

/** The result of the last command sent to several targets, or of Stop all. */
export type BatchFeedback =
  | { kind: "applied"; label: string; instance: EffectInstance; report: BatchReport }
  | { kind: "stopped"; all: boolean; report: BatchReport }
  | { kind: "failed"; error: Failure };

export class EffectsState {
  catalog = $state<EffectCatalog | null>(null);
  catalogProblem = $state<Failure | null>(null);
  /** The targets the user checked; `targets` is the part a command goes to. */
  selection = $state<OutputTarget[]>([]);
  private anchor: OutputTarget | null = null;
  targets = $derived.by(() => {
    const session = this.view().session;
    return session ? effective(this.selection, session) : this.selection;
  });
  target = $derived<OutputTarget | null>(this.targets.length === 1 ? (this.targets[0] ?? null) : null);
  several = $derived(this.targets.length > 1);
  effect = $state<string | null>(null);
  fields = $state<Fields>({});
  rustProblems = $state<Record<string, string>>({});
  working = $state<"apply" | "stop" | null>(null);
  /** The last Apply or Stop press and the targets it went to; live edits never set it. */
  pressed = $state<{ count: number; key: string } | null>(null);
  batch = $state<BatchFeedback | null>(null);
  private edited = $state<string[]>([]);
  /** Settings whose running values differ between the selected targets; Live stays off until an Apply makes them equal, even after each one is edited. */
  differs = $derived(this.effect !== null ? this.differing(this.effect) : []);
  /** The differing settings the user has not changed yet; each is marked Mixed. */
  mixed = $derived(this.differs.filter((id) => !this.edited.includes(id)));
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
    if (this.several) {
      return applyingText(this.view()) ?? (session && isLive(session) ? null : "PrismForge is not connected. Effects can start again after it reconnects.");
    }
    const device = session && this.target ? deviceOf(session, this.target) : null;
    if (!session || !this.target || !device) {
      return "This device is not in the device list.";
    }
    return applyingText(this.view()) ?? applyBlocked(session, device, this.target.zone);
  });
  allRun = $derived.by(() => {
    const session = this.view().session;
    return (
      session !== null &&
      this.effect !== null &&
      this.targets.length > 0 &&
      this.targets.every((target) => runningOn(session, target)?.effect.effect === this.effect)
    );
  });
  /** True only while a changed setting may be sent to every target without the user pressing Apply. */
  canAutoApply = $derived(
    this.allRun && this.differs.length === 0 && this.blocked === null && this.targets.every((target) => !this.isHeld(target)),
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
    this.selection = [];
    this.anchor = null;
    this.effect = null;
    this.fields = {};
    this.clearResults();
  }

  /** The user left the screen or the session; a pending change is dropped and the target waits for Apply. */
  leave() {
    this.cancelReapply();
    for (const target of this.targets) {
      this.hold(targetHold(target));
    }
  }

  /** Selects only `target`, so its own page shows; this is the single-target flow. */
  select(target: OutputTarget) {
    this.leave();
    this.edited = [];
    this.selection = [target];
    this.anchor = target;
    this.effect = null;
    this.fields = {};
    this.clearResults();
    const running = this.running;
    if (running) {
      this.effect = running.effect.effect;
      this.fields = this.definition ? initialFields(this.definition, running.effect.values) : {};
    }
  }

  toggle(target: OutputTarget) {
    const session = this.view().session;
    if (session) {
      this.anchor = target;
      this.change(toggle(this.targets, target, session));
    }
  }

  extendTo(target: OutputTarget) {
    const session = this.view().session;
    if (session) {
      this.change(addRange(this.targets, this.anchor ?? target, target, session));
    }
  }

  selectAll() {
    const session = this.view().session;
    if (session) {
      this.change(selectAll(session));
    }
  }

  selectOnly(targets: OutputTarget[]) {
    this.change(targets);
  }

  /** Selects `targets` and sends them the effect and values chosen now, not what they run. */
  async applyTo(targets: OutputTarget[]) {
    this.change(targets, false);
    await this.apply();
  }

  clear() {
    this.change([]);
  }

  /** A changed selection keeps the chosen effect; when every target runs one effect, its settings show. */
  private change(selection: OutputTarget[], adopt = true) {
    this.cancelReapply();
    const before = this.targets;
    this.selection = selection;
    for (const target of before.filter((old) => !this.targets.some((now) => sameTarget(now, old)))) {
      this.hold(targetHold(target));
    }
    this.clearResults();
    const session = this.view().session;
    const running = session ? this.targets.map((target) => runningOn(session, target)) : [];
    const first = running[0];
    if (adopt && first && running.every((each) => each?.effect.effect === first.effect.effect)) {
      this.effect = first.effect.effect;
      this.fields = this.definition ? initialFields(this.definition, first.effect.values) : {};
    }
    this.edited = [];
  }

  private differing(effect: string): string[] {
    const session = this.view().session;
    const definition = this.catalog?.effects.find((each) => each.id === effect);
    if (!session || !definition || this.targets.length < 2) {
      return [];
    }
    const values = this.targets
      .map((target) => runningOn(session, target))
      .filter((running) => running?.effect.effect === effect)
      .map((running) => initialFields(definition, running?.effect.values ?? {}));
    return definition.controls
      .map((control) => control.id)
      .filter((id) => new Set(values.map((fields) => JSON.stringify(fields[id]))).size > 1);
  }

  choose(effect: string) {
    this.cancelReapply();
    this.effect = effect;
    this.rustProblems = {};
    const session = this.view().session;
    const running = this.running ?? (session && this.targets[0] ? runningOn(session, this.targets[0]) : null);
    const values = running?.effect.effect === effect ? running.effect.values : {};
    this.fields = this.definition ? initialFields(this.definition, values, this.lastColor) : {};
    this.edited = [];
  }

  setField(id: string, field: Field) {
    this.fields = { ...this.fields, [id]: field };
    if (!this.edited.includes(id)) {
      this.edited = [...this.edited, id];
    }
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
    const targets = this.targets;
    this.held = this.held.filter((key) => !targets.some((target) => key === deviceHold(target.device) || key === targetHold(target)));
    if (this.several) {
      await this.sendBatch(targets);
    } else {
      await this.send();
    }
    this.press(targets);
  }

  async stop() {
    if (this.several) {
      await this.stopBatch(this.targets);
      return;
    }
    const target = this.target;
    if (!target) {
      return;
    }
    this.holdDevice(target.device);
    this.working = "stop";
    const answer = await this.call([target.device], () => api.stopEffect(target));
    this.working = null;
    if (answer.ok) {
      this.changed();
      this.modeResult = null;
      this.effectResult = { kind: "stopped", outcome: answer.value, target, revision: this.view().revision };
    } else {
      this.effectResult = { kind: "failed", error: answer.error };
    }
    this.press([target]);
  }

  private async stopBatch(targets: OutputTarget[]) {
    for (const target of targets) {
      this.holdDevice(target.device);
    }
    this.working = "stop";
    const answer = await this.call(
      targets.map((target) => target.device),
      () => api.stopEffectBatch(targets),
    );
    this.working = null;
    this.afterBatch(answer, (report) => ({ kind: "stopped", all: false, report }));
    this.press(targets);
  }

  /** Stops every PrismForge effect, also on targets that are not selected. */
  async stopAll() {
    const session = this.view().session;
    const devices = session ? [...new Set(session.output.scene.assignments.map((assignment) => assignment.target.device))] : [];
    for (const device of devices) {
      this.holdDevice(device);
    }
    this.working = "stop";
    const answer = await this.call(devices, () => api.stopAllEffects());
    this.working = null;
    this.afterBatch(answer, (report) => ({ kind: "stopped", all: true, report }));
    this.effectResult = null;
    this.press(this.targets);
  }

  /** Sends the last batch again to the targets it skipped, with the same effect. */
  async retry() {
    const batch = this.batch;
    if (batch === null || batch.kind === "failed") {
      return;
    }
    const skipped = batch.report.results
      .filter((result) => typeof result.outcome === "object" && "Skipped" in result.outcome && retryable(result.outcome.Skipped.error))
      .map((result) => result.target);
    if (skipped.length === 0) {
      return;
    }
    if (batch.kind === "stopped") {
      await this.stopBatch(skipped);
      return;
    }
    this.working = "apply";
    const answer = await this.call(
      skipped.map((target) => target.device),
      () => api.applyEffectBatch(skipped, batch.instance),
    );
    this.working = null;
    this.afterBatch(answer, (report) => ({ kind: "applied", label: batch.label, instance: batch.instance, report }));
    this.press(this.targets);
  }

  async selectMode(device: DeviceRef, mode: number, name: string) {
    this.holdDevice(device);
    const answer = await this.call([device], () => api.selectMode(device, mode));
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
    const answer = await this.call([device], () => api.declareExternal(device, label));
    this.afterOwnership(answer);
  }

  async clearExternal(device: DeviceRef) {
    this.holdDevice(device);
    const answer = await this.call([device], () => api.clearExternal(device));
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
    const answer = await this.call([target.device], () => api.applyEffect(target, instance));
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

  /** One batch for every target, with the shared start time the session gives it. */
  private async sendBatch(targets: OutputTarget[]) {
    const instance = this.checked?.instance;
    const definition = this.definition;
    if (targets.length === 0 || !instance || !definition) {
      return;
    }
    this.working = "apply";
    const answer = await this.call(
      targets.map((target) => target.device),
      () => api.applyEffectBatch(targets, instance),
    );
    this.working = null;
    if (!answer.ok && typeof answer.error === "object" && "InvalidEffect" in answer.error && "InvalidValues" in answer.error.InvalidEffect) {
      this.rustProblems = Object.fromEntries(
        answer.error.InvalidEffect.InvalidValues.errors.map((value) => [value.control, problemText(value.problem)]),
      );
    } else if (answer.ok) {
      this.rustProblems = {};
      this.edited = [];
    }
    this.afterBatch(answer, (report) => ({ kind: "applied", label: definition.label, instance, report }));
  }

  private afterBatch(answer: Answer<BatchReport>, feedback: (report: BatchReport) => BatchFeedback) {
    if (answer.ok) {
      this.changed();
      this.modeResult = null;
      this.batch = feedback(answer.value);
    } else {
      this.batch = { kind: "failed", error: answer.error };
    }
  }

  private autoApply() {
    this.reapplyTimer = undefined;
    const targets = this.targets;
    if (!this.canAutoApply || targets.length === 0) {
      return;
    }
    if (targets.some((target) => (this.inFlight[target.device] ?? 0) > 0)) {
      this.reapplyAfterCall = true;
      return;
    }
    if (this.several) {
      this.sendBatch(targets);
    } else {
      this.send();
    }
  }

  private async call<T>(devices: DeviceRef[], command: () => Promise<Answer<T>>): Promise<Answer<T>> {
    const count = (device: DeviceRef, step: number) => (this.inFlight = { ...this.inFlight, [device]: (this.inFlight[device] ?? 0) + step });
    devices.forEach((device) => count(device, 1));
    try {
      return await command();
    } finally {
      devices.forEach((device) => count(device, -1));
      if (this.reapplyAfterCall && devices.every((device) => this.inFlight[device] === 0)) {
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
    this.batch = null;
    this.effectResult = null;
    this.modeResult = null;
    this.externalProblem = null;
  }

  private press(targets: OutputTarget[]) {
    if (targets.length > 0) {
      this.pressed = { count: (this.pressed?.count ?? 0) + 1, key: targetsKey(targets) };
    }
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
