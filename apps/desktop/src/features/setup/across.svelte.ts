import { api, type Failure } from "../../lib/api/client";
import type { AcrossReport } from "../../lib/api/bindings/AcrossReport";
import type { BatchReport } from "../../lib/api/bindings/BatchReport";
import type { EffectCatalog } from "../../lib/api/bindings/EffectCatalog";
import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
import type { Flow } from "../../lib/api/bindings/Flow";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { ShellView } from "../../lib/api/bindings/ShellView";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { isLive } from "../devices/present";
import { applyingText } from "../effects/present";
import { retryable } from "../effects/retry";
import { check, initialFields, type Field, type Fields } from "../effects/values";
import { viewTargets } from "./colors";
import { START_FLOW } from "./flow";
import type { SetupState } from "./state.svelte";

export type AcrossResult =
  | { kind: "applied"; label: string; view: ViewId; flow: Flow; instance: EffectInstance; report: AcrossReport }
  | { kind: "stopped"; report: BatchReport }
  | { kind: "failed"; error: Failure };

/** The effect, its settings and the flow for one picture across a view, and what the last Apply or Stop did. */
export class AcrossState {
  effect = $state<string | null>(null);
  fields = $state<Fields>({});
  flow = $state<Flow>(START_FLOW);
  stage = $state<"draft" | "live">("draft");
  working = $state<"apply" | "stop" | null>(null);
  result = $state<AcrossResult | null>(null);

  definition = $derived.by(() => this.catalog()?.effects.find((effect) => effect.id === this.effect) ?? null);
  checked = $derived(this.definition ? check(this.definition, this.fields) : null);
  /** Targets that run an effect across the view on screen now; Stop goes to them. */
  running = $derived.by(() => {
    const session = this.view().session;
    const view = this.setup.tab;
    return (
      session?.output.scene.assignments
        .filter((assignment) => typeof assignment.mode === "object" && assignment.mode.AcrossSetup.view === view)
        .map((assignment) => assignment.target) ?? []
    );
  });
  blocked = $derived.by(() => {
    const view = this.view();
    const held = applyingText(view);
    if (held) {
      return held;
    }
    if (!view.session || !isLive(view.session)) {
      return "PrismForge is not connected. Effects can start again after it reconnects.";
    }
    if (this.setup.dirty) {
      return "Save the setup first. Effects run across the saved setup, and the draft shows your unsaved drawing.";
    }
    const stored = this.setup.stored?.setup;
    if (!stored || viewTargets(stored, this.setup.tab).length === 0) {
      return `Place and wire a component in the ${this.setup.view?.name ?? ""} view first.`;
    }
    return null;
  });

  constructor(
    private readonly view: () => ShellView,
    private readonly setup: SetupState,
    private readonly catalog: () => EffectCatalog | null,
    private readonly changed: () => void,
  ) {}

  choose(effect: string | null) {
    this.effect = effect;
    this.fields = this.definition ? initialFields(this.definition) : {};
  }

  setField(id: string, field: Field) {
    this.fields = { ...this.fields, [id]: field };
  }

  setFlow(flow: Flow) {
    this.flow = flow;
  }

  async apply(targets: OutputTarget[] | null = null, again: Extract<AcrossResult, { kind: "applied" }> | null = null) {
    const instance = again?.instance ?? this.checked?.instance;
    const definition = this.definition;
    if (!instance || (!again && !definition)) {
      return;
    }
    const view = again?.view ?? this.setup.tab;
    const flow = again?.flow ?? this.flow;
    const label = again?.label ?? `${definition!.label} across the ${this.setup.view?.name ?? ""} view`;
    this.working = "apply";
    const answer = await api.applyAcross(view, flow, instance, targets);
    this.working = null;
    if (answer.ok) {
      this.changed();
      this.result = { kind: "applied", label, view, flow, instance, report: answer.value };
    } else {
      this.result = { kind: "failed", error: answer.error };
    }
  }

  /** Sends the last picture again, on its own view and flow, to the targets a second try can reach. */
  async retry() {
    const last = this.result;
    if (last?.kind !== "applied") {
      return;
    }
    const targets = last.report.report.results
      .filter((result) => typeof result.outcome === "object" && "Skipped" in result.outcome && retryable(result.outcome.Skipped.error))
      .map((result) => result.target);
    if (targets.length > 0) {
      await this.apply(targets, last);
    }
  }

  async stop() {
    const targets = this.running;
    if (targets.length === 0) {
      return;
    }
    this.working = "stop";
    const answer = await api.stopEffectBatch(targets);
    this.working = null;
    if (answer.ok) {
      this.changed();
      this.result = { kind: "stopped", report: answer.value };
    } else {
      this.result = { kind: "failed", error: answer.error };
    }
  }
}
