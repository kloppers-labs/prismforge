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

  private dismissed = $state<string[]>([]);

  /** Failed sleep and session-end runs the user has not seen yet; the start notice shows them. */
  get unseenFailures(): RuleRun[] {
    return (this.view?.rules ?? []).map((rule) => rule.last_run).filter((run): run is RuleRun => run !== null && this.noticeable(run));
  }

  /** Adds the run just published when the store has not recorded it, as when its record ran out of time. */
  unseenWith(published: RuleRun | null): RuleRun[] {
    const recorded = this.unseenFailures;
    const known = (this.view?.rules ?? []).some((rule) => rule.last_run !== null && key(rule.last_run) === (published && key(published)));
    if (published === null || known || !this.noticeable(published) || this.dismissed.includes(key(published))) {
      return recorded;
    }
    return [...recorded.filter((run) => run.trigger !== published.trigger), published];
  }

  async dismiss(runs: RuleRun[]) {
    this.dismissed = [...this.dismissed, ...runs.map(key)];
    await this.markSeen(runs.map((run) => run.trigger));
  }

  private noticeable(run: RuleRun): boolean {
    return !run.seen && (run.trigger === "Sleeps" || run.trigger === "SessionEnds") && runFailed(run);
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

function key(run: RuleRun): string {
  return `${run.trigger}:${run.at_ms}`;
}
