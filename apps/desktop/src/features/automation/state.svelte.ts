import { api, type Failure } from "../../lib/api/client";
import type { AutomationView } from "../../lib/api/bindings/AutomationView";
import type { RuleAction } from "../../lib/api/bindings/RuleAction";
import type { Trigger } from "../../lib/api/bindings/Trigger";

export type RuleFailure = { trigger: Trigger; error: Failure };

/** The rules as Rust reports them; a change is one command, and a refusal is shown, never retried. */
export class AutomationState {
  view = $state<AutomationView | null>(null);
  pending = $state<Trigger | null>(null);
  failure = $state<RuleFailure | null>(null);
  loadFailure = $state<Failure | null>(null);
  private opened = false;

  async load() {
    this.opened = true;
    const answer = await api.automation();
    if (answer.ok) {
      this.view = answer.value;
      this.loadFailure = null;
    } else {
      this.loadFailure = answer.error;
    }
  }

  /** Reads the rules again only once the screen was opened, so a run elsewhere never opens a store. */
  async reload() {
    if (this.opened) {
      await this.load();
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
