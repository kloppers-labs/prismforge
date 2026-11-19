type Entry<T> = { state: T; label: string };

export class History<T> {
  private past: Entry<T>[] = [];
  private future: Entry<T>[] = [];

  constructor(private readonly limit = 100) {}

  get undoLabel(): string | null {
    return this.past.at(-1)?.label ?? null;
  }

  get redoLabel(): string | null {
    return this.future.at(-1)?.label ?? null;
  }

  record(before: T, label: string) {
    this.past = [...this.past, { state: before, label }].slice(-this.limit);
    this.future = [];
  }

  undo(current: T): Entry<T> | null {
    const entry = this.past.at(-1);
    if (!entry) {
      return null;
    }
    this.past = this.past.slice(0, -1);
    this.future = [...this.future, { state: current, label: entry.label }];
    return entry;
  }

  redo(current: T): Entry<T> | null {
    const entry = this.future.at(-1);
    if (!entry) {
      return null;
    }
    this.future = this.future.slice(0, -1);
    this.past = [...this.past, { state: current, label: entry.label }];
    return entry;
  }

  clear() {
    this.past = [];
    this.future = [];
  }
}
