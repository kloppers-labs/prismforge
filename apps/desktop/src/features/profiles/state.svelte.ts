import { api, type Answer, type Failure } from "../../lib/api/client";
import type { ApplyOutcome } from "../../lib/api/bindings/ApplyOutcome";
import type { DeviceRef } from "../../lib/api/bindings/DeviceRef";
import type { ExportReport } from "../../lib/api/bindings/ExportReport";
import type { ImportReport } from "../../lib/api/bindings/ImportReport";
import type { ProfileDetails } from "../../lib/api/bindings/ProfileDetails";
import type { ProfileId } from "../../lib/api/bindings/ProfileId";
import type { Saved } from "../../lib/api/bindings/Saved";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
import type { StoreView } from "../../lib/api/bindings/StoreView";
import { Names } from "../../lib/names";

const REFRESH_DELAY_MS = 150;

export type Outcome =
  | { kind: "applied"; name: string; outcome: ApplyOutcome }
  | { kind: "created"; name: string; saved: Saved }
  | { kind: "updated"; name: string; saved: Saved }
  | { kind: "nothing"; name: string }
  | { kind: "notActive"; name: string }
  | { kind: "exported"; name: string; report: ExportReport }
  | { kind: "exportCancelled" }
  | { kind: "deleted"; name: string }
  | { kind: "duplicated"; name: string }
  | { kind: "failed"; error: Failure };

export type PendingUpdate = { id: ProfileId; name: string; preview: Saved };

/** A rename, export or delete that a context menu asked the profile page to start, with the row to return focus to. */
export type ProfileAsk = { id: ProfileId; action: "rename" | "export" | "delete"; from: HTMLElement | null };

export type Work = "apply" | "save" | "update" | "export" | "import" | "delete" | "duplicate";

export class ProfilesState {
  store = $state<StoreView | null>(null);
  storeProblem = $state<Failure | null>(null);
  selected = $state<ProfileId | null>(null);
  details = $state<ProfileDetails | null>(null);
  detailsProblem = $state<Failure | null>(null);
  working = $state<Work | null>(null);
  outcome = $state<Outcome | null>(null);
  imported = $state<ImportReport | null>(null);
  importProblem = $state<Failure | null>(null);
  pendingUpdate = $state<PendingUpdate | null>(null);
  asked = $state.raw<ProfileAsk | null>(null);
  /** Rises when the session ends or the store kind changes; results and drafts of the old scope go. */
  scope = $state(0);
  names = $derived(new Names(this.store?.names ?? []));
  active = $derived(this.store?.summary.profiles.find((profile) => profile.id === this.store?.summary.active) ?? null);

  private refreshTimer: ReturnType<typeof setTimeout> | undefined;

  async refresh() {
    clearTimeout(this.refreshTimer);
    const answer = await api.profileStore();
    if (!answer.ok) {
      this.store = null;
      this.storeProblem = answer.error;
      return;
    }
    this.storeProblem = null;
    if (this.store !== null && this.store.kind !== answer.value.kind) {
      this.endScope();
    }
    this.store = answer.value;
    const known = answer.value.summary.profiles.some((profile) => profile.id === this.selected);
    if (this.selected !== null && !known) {
      this.selected = null;
      this.details = null;
    }
    if (this.selected !== null) {
      await this.loadDetails(this.selected);
    }
  }

  refreshSoon() {
    clearTimeout(this.refreshTimer);
    this.refreshTimer = setTimeout(() => this.refresh(), REFRESH_DELAY_MS);
  }

  endScope() {
    this.scope += 1;
    this.asked = null;
    this.outcome = null;
    this.imported = null;
    this.importProblem = null;
    this.pendingUpdate = null;
  }

  forget() {
    clearTimeout(this.refreshTimer);
    this.store = null;
    this.storeProblem = null;
    this.selected = null;
    this.details = null;
    this.pendingUpdate = null;
    this.asked = null;
  }

  async select(id: ProfileId) {
    this.asked = null;
    this.selected = id;
    this.details = null;
    await this.loadDetails(id);
  }

  async show(id: ProfileId) {
    if (this.selected !== id || this.details?.id !== id) {
      await this.select(id);
    }
  }

  async ask(id: ProfileId, action: ProfileAsk["action"], from: HTMLElement | null) {
    await this.show(id);
    if (this.selected === id && this.details?.id === id) {
      this.asked = { id, action, from };
    }
  }

  nameOf(id: ProfileId): string {
    return this.store?.summary.profiles.find((profile) => profile.id === id)?.name ?? this.details?.name ?? id;
  }

  async apply(id: ProfileId) {
    const name = this.nameOf(id);
    const answer = await this.run("apply", () => api.applyProfile(id));
    if (answer === null) {
      return;
    }
    this.outcome = answer.ok ? { kind: "applied", name, outcome: answer.value } : { kind: "failed", error: answer.error };
    await this.refresh();
  }

  async saveNew(name: string): Promise<boolean> {
    const answer = await this.run("save", () => api.createProfile(name));
    if (answer === null) {
      return false;
    }
    if (!answer.ok) {
      this.outcome = { kind: "failed", error: answer.error };
      return false;
    }
    this.outcome = { kind: "created", name, saved: answer.value };
    this.selected = answer.value.id;
    await this.refresh();
    return true;
  }

  /** Shows the change list first; an update that would change nothing writes nothing. */
  async previewUpdate(id: ProfileId) {
    const name = this.nameOf(id);
    const answer = await this.run("update", () => api.previewProfileUpdate(id));
    if (answer === null) {
      return;
    }
    if (!answer.ok) {
      this.outcome = this.updateFailure(name, answer.error);
      return;
    }
    const preview = answer.value;
    if (preview.changes.length === 0 && preview.frame_rate === null) {
      this.outcome = { kind: "nothing", name };
      return;
    }
    this.outcome = null;
    this.pendingUpdate = { id, name, preview };
  }

  cancelUpdate() {
    this.pendingUpdate = null;
  }

  async confirmUpdate() {
    const pending = this.pendingUpdate;
    if (!pending) {
      return;
    }
    this.pendingUpdate = null;
    const answer = await this.run("update", () => api.updateProfile(pending.id));
    if (answer === null) {
      return;
    }
    this.outcome = answer.ok
      ? { kind: "updated", name: pending.name, saved: answer.value }
      : this.updateFailure(pending.name, answer.error);
    await this.refresh();
  }

  async rename(id: ProfileId, name: string): Promise<Failure | null> {
    const answer = await api.renameProfile(id, name);
    if (answer.ok) {
      await this.refresh();
      return null;
    }
    return answer.error;
  }

  async duplicate(id: ProfileId) {
    const name = this.nameOf(id);
    const answer = await this.run("duplicate", () => api.duplicateProfile(id));
    if (answer === null) {
      return;
    }
    this.outcome = answer.ok ? { kind: "duplicated", name } : { kind: "failed", error: answer.error };
    if (answer.ok) {
      this.selected = answer.value;
    }
    await this.refresh();
  }

  async remove(id: ProfileId) {
    const name = this.nameOf(id);
    const answer = await this.run("delete", () => api.deleteProfile(id));
    if (answer === null) {
      return;
    }
    if (answer.ok) {
      this.outcome = { kind: "deleted", name };
      this.selected = null;
      this.details = null;
    } else {
      this.outcome = { kind: "failed", error: answer.error };
    }
    await this.refresh();
  }

  async exportProfile(id: ProfileId) {
    const name = this.nameOf(id);
    const answer = await this.run("export", () => api.exportProfile(id));
    if (answer === null) {
      return;
    }
    if (!answer.ok) {
      this.outcome = { kind: "failed", error: answer.error };
    } else if (answer.value === null) {
      this.outcome = { kind: "exportCancelled" };
    } else {
      this.outcome = { kind: "exported", name, report: answer.value };
    }
  }

  async importProfile() {
    this.importProblem = null;
    const answer = await this.run("import", () => api.importProfile());
    if (answer === null) {
      return;
    }
    if (!answer.ok) {
      this.importProblem = answer.error;
      return;
    }
    if (answer.value === null) {
      return;
    }
    this.imported = answer.value;
    this.selected = answer.value.details.id;
    await this.refresh();
  }

  async dismiss(notice: StoreNotice) {
    const scope = this.scope;
    const answer = await api.dismissStoreNotice(notice);
    if (!answer.ok && scope === this.scope) {
      this.outcome = { kind: "failed", error: answer.error };
    }
    await this.refresh();
  }

  async setDeviceName(device: DeviceRef, name: string | null): Promise<Failure | null> {
    return this.afterNaming(await api.setDeviceName(device, name));
  }

  async setZoneName(device: DeviceRef, zone: number, name: string | null): Promise<Failure | null> {
    return this.afterNaming(await api.setZoneName(device, zone, name));
  }

  private async afterNaming(answer: Answer<null>): Promise<Failure | null> {
    if (!answer.ok) {
      return answer.error;
    }
    this.refresh();
    return null;
  }

  private updateFailure(name: string, error: Failure): Outcome {
    if (typeof error === "object" && "Store" in error && "NotActive" in error.Store) {
      return { kind: "notActive", name };
    }
    return { kind: "failed", error };
  }

  private async loadDetails(id: ProfileId) {
    const answer = await api.profileDetails(id);
    if (this.selected !== id) {
      return;
    }
    if (answer.ok) {
      this.details = answer.value;
      this.detailsProblem = null;
    } else {
      this.details = null;
      this.detailsProblem = answer.error;
    }
  }

  /** `null` when the session or the store changed meanwhile: the answer belongs to an ended scope. */
  private async run<T>(work: Work, action: () => Promise<Answer<T>>): Promise<Answer<T> | null> {
    const scope = this.scope;
    this.working = work;
    try {
      const answer = await action();
      return scope === this.scope ? answer : null;
    } finally {
      this.working = null;
    }
  }
}
