import { api, type Failure } from "../../lib/api/client";
import type { ComponentId } from "../../lib/api/bindings/ComponentId";
import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
import type { Setup } from "../../lib/api/bindings/Setup";
import type { SetupStoreView } from "../../lib/api/bindings/SetupStoreView";
import type { StoreKind } from "../../lib/api/bindings/StoreKind";
import type { ViewId } from "../../lib/api/bindings/ViewId";
import { reference } from "../devices/present";
import { problems } from "./bounds";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { OutputId } from "../../lib/api/bindings/OutputId";
import {
  VIEWS,
  addComponent,
  addOutline,
  flip,
  moveBy,
  moveInChain,
  moveTo,
  outlineOf,
  place,
  placementOf,
  removeComponent,
  removeOutline,
  rename,
  rotateBy,
  setFirstLed,
  setFrame,
  setGeometry,
  setOutline,
  setRotation,
  setSize,
  unplace,
  unwire,
  wire,
  withViews,
  type Joining,
  type NewComponent,
  type OutputChoice,
} from "./edits";
import type { ShapePart } from "./library";
import type { Template } from "./templates";
import { OUTLINES } from "./library";
import type { Outline } from "../../lib/api/bindings/Outline";
import type { OutlineKind } from "../../lib/api/bindings/OutlineKind";
import { nameOf, selectionName } from "./present";
import { History } from "./history";

type Snapshot = { setup: Setup; selection: ComponentId[] };

type Parked = { stored: SetupStoreView; draft: Setup; selection: ComponentId[]; history: History<Snapshot> };

const EMPTY: Setup = { components: [], outputs: [], views: [] };

function same(left: Setup, right: Setup): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

/** The smallest free id with this prefix; ids stay unique across components, outputs and views. */
export function freeId(setup: Setup, prefix: "c-" | "o-" | "u-"): string {
  const used = new Set([
    ...setup.components.map((each) => each.id),
    ...setup.outputs.map((each) => each.id),
    ...setup.views.map((each) => each.id),
    ...setup.views.flatMap((view) => view.outlines.map((each) => each.id)),
  ]);
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
  /** A dialog that a menu item asked the editor to open, as its button would. */
  asked = $state<{ kind: "delete" | "group"; from?: HTMLElement } | null>(null);

  dirty = $derived(this.stored !== null && !same(this.draft, withViews(this.stored.setup)));
  view = $derived(this.draft.views.find((view) => view.id === this.tab) ?? null);
  placed = $derived(this.view?.placements.map((placement) => placement.component) ?? []);
  drawn = $derived(this.view?.outlines.map((outline) => outline.id) ?? []);
  issues = $derived.by(() => {
    const session = this.session();
    return session ? problems(this.draft, session) : [];
  });

  private history = new History<Snapshot>();
  private parked = new Map<StoreKind, Parked>();
  private key = "";
  private loading: Promise<void> | null = null;
  private again = false;

  constructor(private readonly session: () => SessionSnapshot | null) {}

  /** Reads the stored setup when the devices or the store change; an unsaved drawing of the other store waits for its return. */
  follow(session: SessionSnapshot | null) {
    const key = signature(session);
    if (key !== this.key) {
      this.key = key;
      void this.load();
    }
  }

  /** A load asked for while a read runs reads again once that read ends, so a newer setup is never dropped. */
  load(): Promise<void> {
    if (this.loading) {
      this.again = true;
      return this.loading;
    }
    this.loading = this.read().finally(() => {
      this.loading = null;
      if (this.again) {
        this.again = false;
        void this.load();
      }
    });
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
    const answer = await api.saveSetup($state.snapshot(this.draft));
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

  moveSelection(dx: number, dy: number) {
    const ids = this.selection;
    if (ids.length > 0) {
      this.edit(`Move ${selectionName(this.draft, ids)}`, (setup) => moveBy(setup, this.tab, ids, dx, dy));
    }
  }

  turnSelection(steps: number) {
    const ids = this.selection;
    if (ids.length > 0) {
      this.edit(`Turn ${selectionName(this.draft, ids)}`, (setup) => rotateBy(setup, this.tab, ids, steps));
    }
  }

  flipSelection() {
    const ids = this.selection;
    if (ids.length > 0) {
      this.edit(`Flip ${selectionName(this.draft, ids)}`, (setup) => flip(setup, this.tab, ids));
    }
  }

  takeOut(ids: ComponentId[] = this.selection) {
    if (ids.length > 0) {
      const view = this.view?.name ?? "";
      this.edit(
        `Take ${selectionName(this.draft, ids)} out of the ${view} view`,
        (setup) => ids.reduce((next, id) => removeOutline(unplace(next, this.tab, id), this.tab, id), setup),
        this.selection.filter((id) => !ids.includes(id)),
      );
    }
  }

  setPlace(id: ComponentId, field: "x" | "y" | "size" | "rotation", value: number) {
    if (!Number.isFinite(value)) {
      return;
    }
    const at = placementOf(this.draft, this.tab, id);
    if (!at) {
      return;
    }
    const name = nameOf(this.draft, id);
    if (field === "rotation") {
      this.edit(`Turn ${name}`, (setup) => setRotation(setup, this.tab, id, value));
    } else if (field === "size") {
      this.edit(`Resize ${name}`, (setup) => setSize(setup, this.tab, id, value));
    } else {
      this.edit(`Move ${name}`, (setup) => moveTo(setup, this.tab, id, field === "x" ? value : at.x, field === "y" ? value : at.y));
    }
  }

  rename(id: ComponentId, name: string) {
    this.edit(`Rename ${nameOf(this.draft, id)}`, (setup) => rename(setup, id, name));
  }

  reshape(id: ComponentId, geometry: Geometry, what: string) {
    this.edit(`${what} of ${nameOf(this.draft, id)}`, (setup) => setGeometry(setup, id, geometry));
  }

  /** Adds the parts of one shape at one spot as one step; later parts follow the first on the wire. */
  addShape(name: string, parts: ShapePart[], at: { x: number; y: number; size: number }, wiring: { choice: OutputChoice; joining: Joining } | null) {
    const ids: ComponentId[] = [];
    this.edit(
      `Place ${name}`,
      (setup) =>
        parts.reduce((next, part, index) => {
          const id = freeId(next, "c-");
          ids.push(id);
          const added = addComponent(next, this.tab, { id, name: `${name}${part.suffix}`, geometry: part.geometry, ...(part.frame ? { frame: part.frame } : {}) }, { ...at, size: Math.round(at.size * part.scale) });
          if (!wiring) {
            return added;
          }
          const joining = index === 0 ? wiring.joining : "chain";
          return wire(added, id, wiring.choice, joining, freeId(added, "o-"));
        }, setup),
      ids,
    );
  }

  /** Adds a template's components to its view as one step; none of them is wired. */
  addTemplate(template: Template) {
    const ids: ComponentId[] = [];
    this.show(template.view);
    this.edit(
      `Add the ${template.label} template`,
      (setup) =>
        template.slots.reduce((next, slot) => {
          const id = freeId(next, "c-");
          ids.push(id);
          return addComponent(next, template.view, { id, name: slot.name, geometry: slot.geometry }, slot);
        }, template.outlines.reduce((next, backdrop) => addOutline(next, template.view, { ...backdrop, id: freeId(next, "u-"), rotation: 0 }), setup)),
      ids,
    );
  }

  placeHere(id: ComponentId, at: { x: number; y: number; size?: number; rotation?: number }) {
    this.edit(`Place ${nameOf(this.draft, id)} in the ${this.view?.name ?? ""} view`, (setup) => place(setup, this.tab, id, at), [id]);
  }

  wire(id: ComponentId, choice: OutputChoice, joining: Joining) {
    this.edit(`Wire ${nameOf(this.draft, id)}`, (setup) => wire(setup, id, choice, joining, freeId(setup, "o-")));
  }

  unwire(id: ComponentId) {
    this.edit(`Unwire ${nameOf(this.draft, id)}`, (setup) => unwire(setup, id));
  }

  moveInChain(id: ComponentId, delta: number) {
    this.edit(`Move ${nameOf(this.draft, id)} ${delta < 0 ? "earlier" : "later"} in its chain`, (setup) => moveInChain(setup, id, delta));
  }

  setFirstLed(output: OutputId, first: number) {
    this.edit("Change the first LED of an output", (setup) => setFirstLed(setup, output, first));
  }

  remove(id: ComponentId) {
    this.edit(
      `Delete ${nameOf(this.draft, id)}`,
      (setup) => removeComponent(setup, id),
      this.selection.filter((each) => each !== id),
    );
  }

  addOutline(kind: OutlineKind) {
    const shape = OUTLINES.find((each) => each.kind === kind)!;
    let id = "";
    this.edit(
      `Add the ${shape.name.toLowerCase()} outline`,
      (setup) => {
        id = freeId(setup, "u-");
        return addOutline(setup, this.tab, { id, name: shape.name, kind, x: shape.x, y: shape.y, width: shape.width, height: shape.height, rotation: 0 });
      },
      [],
    );
    this.selection = [id];
  }

  setOutline(id: string, patch: Partial<Omit<Outline, "id">>, what: string) {
    this.edit(`${what} ${nameOf(this.draft, id)}`, (setup) => setOutline(setup, this.tab, id, patch));
  }

  removeOutline(id: string) {
    this.edit(
      `Delete ${nameOf(this.draft, id)}`,
      (setup) => removeOutline(setup, this.tab, id),
      this.selection.filter((each) => each !== id),
    );
  }

  setFrame(id: ComponentId, frame: { width: number; height: number } | null, what: string) {
    this.edit(`${what} of ${nameOf(this.draft, id)}`, (setup) => setFrame(setup, id, frame));
  }

  private keepSelection() {
    this.selection = this.selection.filter((id) => placementOf(this.draft, this.tab, id) !== null || outlineOf(this.draft, this.tab, id) !== null);
  }

  private syncHistory() {
    this.undoLabel = this.history.undoLabel;
    this.redoLabel = this.history.redoLabel;
  }
}
