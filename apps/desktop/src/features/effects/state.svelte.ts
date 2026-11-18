import { api, type Failure } from "../../lib/api/client";
import type { DeviceRef } from "../../lib/api/bindings/DeviceRef";
import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { RestoreOutcome } from "../../lib/api/bindings/RestoreOutcome";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import { capitalise, serverName } from "../connection/wording";
import { deviceOf, runningOn, targetName } from "./present";
import { check, initialFields, problemText, type Field, type Fields } from "./values";

const REAPPLY_DELAY_MS = 300;

export type EffectFeedback =
  | { kind: "applied"; effect: string; target: string }
  | { kind: "stopped"; outcome: RestoreOutcome | null }
  | { kind: "failed"; error: Failure };

export type ActionFeedback = { kind: "done"; text: string } | { kind: "failed"; error: Failure };

export class EffectsState {
  catalog = $state<EffectCatalog | null>(null);
  catalogProblem = $state<Failure | null>(null);
  target = $state<OutputTarget | null>(null);
  effect = $state<string | null>(null);
  fields = $state<Fields>({});
  rustProblems = $state<Record<string, string>>({});
  working = $state<"apply" | "stop" | null>(null);
  feedback = $state<EffectFeedback | null>(null);
  modeFeedback = $state<ActionFeedback | null>(null);
  externalFeedback = $state<ActionFeedback | null>(null);
  frameRateFeedback = $state<ActionFeedback | null>(null);

  definition = $derived(this.catalog?.effects.find((effect) => effect.id === this.effect) ?? null);
  checked = $derived(this.definition ? check(this.definition, this.fields) : null);

  private reapplyTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly session: () => SessionSnapshot | null) {}

  async load() {
    if (this.catalog) {
      return;
    }
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
    this.target = null;
    this.effect = null;
    this.fields = {};
    this.clearFeedback();
  }

  select(target: OutputTarget) {
    this.reset();
    this.target = target;
    const session = this.session();
    const running = session ? runningOn(session, target) : null;
    if (running) {
      this.effect = running.effect.effect;
      this.fields = this.definition ? initialFields(this.definition, running.effect.values) : {};
    }
  }

  choose(effect: string) {
    this.cancelReapply();
    this.effect = effect;
    this.rustProblems = {};
    const running = this.runningAssignment();
    const values = running?.effect.effect === effect ? running.effect.values : {};
    this.fields = this.definition ? initialFields(this.definition, values) : {};
  }

  setField(id: string, field: Field) {
    this.fields = { ...this.fields, [id]: field };
    const { [id]: _, ...others } = this.rustProblems;
    this.rustProblems = others;
    if (this.isLive()) {
      this.cancelReapply();
      this.reapplyTimer = setTimeout(() => this.apply(), REAPPLY_DELAY_MS);
    }
  }

  /** True when the chosen effect already runs on the target, so changes re-apply without a restart. */
  isLive(): boolean {
    const running = this.runningAssignment();
    return running !== null && running.effect.effect === this.effect;
  }

  async apply() {
    this.cancelReapply();
    const target = this.target;
    const instance = this.checked?.instance;
    const label = this.definition?.label;
    if (!target || !instance || !label) {
      return;
    }
    this.working = "apply";
    const answer = await api.applyEffect(target, instance);
    this.working = null;
    if (answer.ok) {
      this.rustProblems = {};
      this.feedback = { kind: "applied", effect: label, target: this.targetLabel(target) };
      return;
    }
    const error = answer.error;
    if (typeof error === "object" && "InvalidEffect" in error && "InvalidValues" in error.InvalidEffect) {
      this.rustProblems = Object.fromEntries(
        error.InvalidEffect.InvalidValues.errors.map((value) => [value.control, problemText(value.problem)]),
      );
    }
    this.feedback = { kind: "failed", error };
  }

  async stop() {
    this.cancelReapply();
    const target = this.target;
    if (!target) {
      return;
    }
    this.working = "stop";
    const answer = await api.stopEffect(target);
    this.working = null;
    this.feedback = answer.ok ? { kind: "stopped", outcome: answer.value } : { kind: "failed", error: answer.error };
  }

  async selectMode(device: DeviceRef, mode: number, name: string) {
    this.cancelReapply();
    const answer = await api.selectMode(device, mode);
    this.modeFeedback = answer.ok
      ? { kind: "done", text: `Hardware mode ${name} selected. ${capitalise(this.server())} accepted the request. PrismForge cannot confirm what the device shows.` }
      : { kind: "failed", error: answer.error };
  }

  async declareExternal(device: DeviceRef, label: string) {
    this.cancelReapply();
    const answer = await api.declareExternal(device, label);
    this.externalFeedback = answer.ok ? null : { kind: "failed", error: answer.error };
  }

  async clearExternal(device: DeviceRef) {
    const answer = await api.clearExternal(device);
    this.externalFeedback = answer.ok ? null : { kind: "failed", error: answer.error };
  }

  async setFrameRate(requested: number) {
    const answer = await api.setFrameRate(requested);
    if (!answer.ok) {
      this.frameRateFeedback = { kind: "failed", error: answer.error };
      return;
    }
    const used = answer.value;
    this.frameRateFeedback = {
      kind: "done",
      text:
        used === requested
          ? `PrismForge uses ${used} frames per second.`
          : `PrismForge uses ${used} frames per second, the nearest allowed rate.`,
    };
  }

  private server(): string {
    const session = this.session();
    return session ? serverName(session) : "OpenRGB";
  }

  private runningAssignment() {
    const session = this.session();
    return session && this.target ? runningOn(session, this.target) : null;
  }

  private targetLabel(target: OutputTarget): string {
    const session = this.session();
    const device = session ? deviceOf(session, target) : null;
    return device ? targetName(device, target.zone) : target.device;
  }

  private clearFeedback() {
    this.rustProblems = {};
    this.feedback = null;
    this.modeFeedback = null;
    this.externalFeedback = null;
  }

  private cancelReapply() {
    clearTimeout(this.reapplyTimer);
    this.reapplyTimer = undefined;
  }
}
