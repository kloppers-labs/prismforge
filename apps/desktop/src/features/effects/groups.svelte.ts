import { api, type Answer, type Failure } from "../../lib/api/client";
import type { GroupId } from "../../lib/api/bindings/GroupId";
import type { GroupView } from "../../lib/api/bindings/GroupView";
import type { GroupsView } from "../../lib/api/bindings/GroupsView";
import type { MemberStatus } from "../../lib/api/bindings/MemberStatus";
import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
import { fileProblemText } from "../../lib/errors";
import { reference } from "../devices/present";
import { choosable, selectAll } from "./selection";
import type { EffectsState } from "./state.svelte";

/** The built-in group: every target that can take an effect today; it is computed, never stored. */
export const EVERYTHING = "everything";

export type Picked = { targets: OutputTarget[]; left: MemberStatus[] };

/** A dialog a menu item or a button asked the groups panel to open. */
export type GroupAsk = { kind: "create" | "add"; from?: HTMLElement } | { kind: "rename" | "delete"; group: GroupView; from?: HTMLElement };

export function stateText(member: MemberStatus): string {
  switch (member.state) {
    case "Resolved":
      return "here";
    case "Unavailable":
      return "not reported right now";
    case "Missing":
      return "not found today";
    case "Ambiguous":
      return "cannot be told apart from an identical device";
    case "Changed":
      return member.zone_name ? `its zone is no longer ${member.zone_name}` : "its zone changed";
  }
}

export function storedLabel(member: MemberStatus): string {
  return member.zone_name ? `${member.label}, ${member.zone_name}` : member.label;
}

export function groupsNoticeText(notice: StoreNotice): string {
  const event = notice.event;
  if ("RestoredFromBackup" in event) {
    const problem = event.RestoredFromBackup.problem;
    const why = problem === null ? `${notice.file} was missing.` : `PrismForge could not read ${notice.file}. ${fileProblemText(problem)}`;
    return `${why} It used the backup, so your last change to a group may be missing.`;
  }
  if ("StartedEmpty" in event) {
    return `PrismForge could not read ${notice.file} or its backup, so it starts without groups. The damaged files are kept.`;
  }
  if ("LeftUntouched" in event) {
    return `${notice.file} needs a newer PrismForge. This version shows no groups and does not change the file.`;
  }
  return `PrismForge could not save ${event.WriteFailed.failure.file}. The previous version is still in place.`;
}

/** Changes when what a member can resolve to changes, so the groups are read again. */
function signature(session: SessionSnapshot | null): string {
  if (!session) {
    return "none";
  }
  const devices = session.devices.map(
    (device) => `${reference(device) ?? "?"}:${device.available}:${device.descriptor.zones.map((zone) => zone.name).join("/")}`,
  );
  return `${session.backend.origin}|${devices.join(",")}`;
}

/** The groups of the store in use; PrismForge reads them again when the devices change. */
export class GroupsState {
  view = $state<GroupsView | null>(null);
  problem = $state<Failure | null>(null);
  working = $state(false);
  /** What the last Select did, said once in the panel's status line. */
  note = $state<string | null>(null);
  asked = $state<GroupAsk | null>(null);
  private key = "";

  follow(session: SessionSnapshot | null) {
    const key = signature(session);
    if (key !== this.key) {
      this.key = key;
      void this.load();
    }
  }

  async load() {
    this.take(await api.groups());
  }

  /** The members that resolve and can take an effect now, and the ones left out. */
  pick(group: GroupView | typeof EVERYTHING, session: SessionSnapshot): Picked {
    if (group === EVERYTHING) {
      return { targets: selectAll(session), left: [] };
    }
    const usable = (member: MemberStatus) => member.state === "Resolved" && choosable(session, member.target);
    return {
      targets: group.members.filter(usable).map((member) => member.target),
      left: group.members.filter((member) => !usable(member)),
    };
  }

  /** Selects the members that can take an effect and says which members were left out. */
  select(group: GroupView | typeof EVERYTHING, session: SessionSnapshot, effects: EffectsState) {
    const picked = this.pick(group, session);
    effects.selectOnly(picked.targets);
    const title = group === EVERYTHING ? "Everything" : group.name;
    const count = picked.targets.length === 1 ? "1 target" : `${picked.targets.length} targets`;
    const left = picked.left.map((member) => `${storedLabel(member)} (${member.state === "Resolved" ? "cannot take an effect now" : stateText(member)})`);
    this.note = `Selected ${title}: ${count}.${left.length === 0 ? "" : ` Left out: ${left.join(", ")}.`}`;
  }

  create(name: string, targets: OutputTarget[]): Promise<boolean> {
    return this.run(() => api.createGroup(name, targets));
  }

  rename(id: GroupId, name: string): Promise<boolean> {
    return this.run(() => api.renameGroup(id, name));
  }

  add(id: GroupId, targets: OutputTarget[]): Promise<boolean> {
    return this.run(() => api.addGroupMembers(id, targets));
  }

  remove(id: GroupId, target: OutputTarget): Promise<boolean> {
    return this.run(() => api.removeGroupMember(id, target));
  }

  delete(id: GroupId): Promise<boolean> {
    return this.run(() => api.deleteGroup(id));
  }

  private async run(command: () => Promise<Answer<GroupsView>>): Promise<boolean> {
    this.working = true;
    const answer = await command();
    this.working = false;
    return this.take(answer);
  }

  private take(answer: Answer<GroupsView>): boolean {
    if (answer.ok) {
      this.view = answer.value;
      this.problem = null;
      return true;
    }
    this.problem = answer.error;
    return false;
  }
}
