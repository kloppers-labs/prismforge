import { api, type Failure } from "../../lib/api/client";
import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupStoreView } from "../../lib/api/bindings/SetupStoreView";
import type { StoreKind } from "../../lib/api/bindings/StoreKind";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { reference } from "../devices/present";
import { problems } from "./bounds";
import { VIEWS, placementOf, withViews } from "./edits";
import { History } from "./history";

type Snapshot = { setup: Setup; selection: ComponentId[] };

type Parked = { stored: SetupStoreView; draft: Setup; selection: ComponentId[]; history: History<Snapshot> };

const EMPTY: Setup = { components: [], outputs: [], views: [] };

function same(left: Setup, right: Setup): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

/** The smallest free id with this prefix; ids stay unique across components, outputs and views. */
export function freeId(setup: Setup, prefix: "c-" | "o-"): string {
  const used = new Set([...setup.components.map((each) => each.id), ...setup.outputs.map((each) => each.id), ...setup.views.map((each) => each.id)]);
  let number = 1;
  while (used.has(`${prefix}${number}`)) {
    number += 1;
  }
  return `${prefix}${number}`;
}

/** Changes when the devices a setup output can match change, so the stored view is read again. */
function signature(session: SessionSnapshot | null): string {
  if (!session) {
    return "none";
  }
  const devices = session.devices.map((device) => `${reference(device) ?? "?"}:${device.available}:${device.descriptor.zones.map((zone) => `${zone.name}/${zone.led_count}`).join(",")}`);
  return `${session.backend.origin}|${devices.join(";")}`;
}

/**
 * The setup as stored and the drawing the user edits. Edits change only the drawing until Save setup;
 * undo and redo step through the drawing and never send anything to a device.
 */
export class SetupState {
  stored = $state<SetupStoreView | null>(null);
  loadProblem = $state<Failure | null>(null);
  draft = $state<Setup>(withViews(EMPTY));
  tab = $state<ViewId>(VIEWS.pc);
  selection = $state<ComponentId[]>([]);
  saving = $state(false);
  saveProblem = $state<Failure | null>(null);
  justSaved = $state(false);
  undoLabel = $state<string | null>(null);
  redoLabel = $state<string | null>(null);

  dirty = $derived(this.stored !== null && !same(this.draft, withViews(this.stored.setup)));
  view = $derived(this.draft.views.find((view) => view.id === this.tab) ?? null);
  placed = $derived(this.view?.placements.map((placement) => placement.component) ?? []);
  issues = $derived.by(() => {
    const session = this.session();
    return session ? problems(this.draft, session) : [];
  });

  private history = new History<Snapshot>();
  private parked = new Map<StoreKind, Parked>();
  private key = "";
  private loading: Promise<void> | null = null;

  constructor(private readonly session: () => SessionSnapshot | null) {}

  /** Reads the stored setup when the devices or the store change; an unsaved drawing of the other store waits for its return. */
  follow(session: SessionSnapshot | null) {
    const key = signature(session);
    if (key !== this.key) {
      this.key = key;
      void this.load();
    }
  }

  load(): Promise<void> {
    this.loading ??= this.read().finally(() => (this.loading = null));
    return this.loading;
  }

  private async read() {
    const answer = await api.setup();
    if (!answer.ok) {
      this.loadProblem = answer.error;
      return;
    }
    this.loadProblem = null;
    const next = answer.value;
    const before = this.stored;
    if (before && before.kind !== next.kind) {
      this.parked.set(before.kind, { stored: before, draft: this.draft, selection: this.selection, history: this.history });
      const back = this.parked.get(next.kind);
      this.parked.delete(next.kind);
      const keep = back !== undefined && !same(back.draft, withViews(back.stored.setup));
      this.stored = next;
      this.draft = keep ? back.draft : withViews(next.setup);
      this.selection = keep ? back.selection : [];
      this.history = keep ? back.history : new History<Snapshot>();
      this.syncHistory();
      return;
    }
    const clean = before === null || same(this.draft, withViews(before.setup));
    this.stored = next;
    if (clean) {
      this.draft = withViews(next.setup);
      this.keepSelection();
    }
  }

  /** One finished edit is one undo step; an edit that changes nothing adds none. */
  edit(label: string, change: (setup: Setup) => Setup, selection: ComponentId[] = this.selection) {
    const next = change(this.draft);
    if (same(next, this.draft) && selection.join() === this.selection.join()) {
      return;
    }
    this.history.record({ setup: this.draft, selection: this.selection }, label);
    this.draft = next;
    this.selection = selection;
    this.justSaved = false;
    this.syncHistory();
  }

  undo() {
    const entry = this.history.undo({ setup: this.draft, selection: this.selection });
    if (entry) {
      this.draft = entry.state.setup;
      this.selection = entry.state.selection;
      this.justSaved = false;
    }
    this.syncHistory();
  }

  redo() {
    const entry = this.history.redo({ setup: this.draft, selection: this.selection });
    if (entry) {
      this.draft = entry.state.setup;
      this.selection = entry.state.selection;
      this.justSaved = false;
    }
    this.syncHistory();
  }

  async save() {
    if (!this.dirty || this.saving) {
      return;
    }
    this.saving = true;
    this.saveProblem = null;
    const answer = await api.saveSetup(this.draft);
    this.saving = false;
    if (!answer.ok) {
      this.saveProblem = answer.error;
      return;
    }
    this.stored = answer.value;
    this.draft = withViews(answer.value.setup);
    this.justSaved = true;
    this.keepSelection();
  }

  discard() {
    if (!this.stored) {
      return;
    }
    this.draft = withViews(this.stored.setup);
    this.history.clear();
    this.saveProblem = null;
    this.keepSelection();
    this.syncHistory();
  }

  show(tab: ViewId) {
    this.tab = tab;
    this.keepSelection();
  }

  select(ids: ComponentId[]) {
    this.selection = ids.filter((id, index) => ids.indexOf(id) === index);
  }

  toggle(id: ComponentId) {
    this.select(this.selection.includes(id) ? this.selection.filter((each) => each !== id) : [...this.selection, id]);
  }

  private keepSelection() {
    this.selection = this.selection.filter((id) => placementOf(this.draft, this.tab, id) !== null);
  }

  private syncHistory() {
    this.undoLabel = this.history.undoLabel;
    this.redoLabel = this.history.redoLabel;
  }
}
