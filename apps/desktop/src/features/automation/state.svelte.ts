import { api, type Failure } from "../../lib/api/client";
import type { AutomationView } from "../../lib/api/bindings/AutomationView";
import type { RuleAction } from "../../lib/api/bindings/RuleAction";
import type { RuleRun } from "../../lib/api/bindings/RuleRun";
import type { Trigger } from "../../lib/api/bindings/Trigger";
import { runFailed } from "./wording";

export type RuleFailure = { trigger: Trigger; error: Failure };

/** The rules as Rust reports them; a change is one command, and a refusal is shown, never retried. */
export class AutomationState {
  view = $state<AutomationView | null>(null);
  pending = $state<Trigger | null>(null);
  failure = $state<RuleFailure | null>(null);
  loadFailure = $state<Failure | null>(null);

  /** Failed sleep and session-end runs the user has not seen yet; the start notice shows them. */
  get unseenFailures(): RuleRun[] {
    return (this.view?.rules ?? [])
      .map((rule) => rule.last_run)
      .filter((run): run is RuleRun => run !== null && !run.seen && (run.trigger === "Sleeps" || run.trigger === "SessionEnds") && runFailed(run));
  }

  async load() {
    const answer = await api.automation();
    if (answer.ok) {
      this.view = answer.value;
      this.loadFailure = null;
    } else {
      this.loadFailure = answer.error;
    }
  }

  async reload() {
    await this.load();
  }

  async markSeen(triggers: Trigger[]) {
    if (triggers.length === 0) {
      return;
    }
    const answer = await api.markRunsSeen(triggers);
    if (answer.ok) {
      this.view = answer.value;
    }
  }

  async setRule(trigger: Trigger, enabled: boolean, action: RuleAction) {
    this.pending = trigger;
    this.failure = null;
    const answer = await api.setRule(trigger, enabled, action);
    this.pending = null;
    if (answer.ok) {
      this.view = answer.value;
    } else {
      this.failure = { trigger, error: answer.error };
    }
  }
}
