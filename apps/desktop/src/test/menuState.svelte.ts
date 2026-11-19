import type { Menu, MenuItem } from "../lib/shell/menu.svelte";

/** A menu whose items a test can change while it is open. */
export class LiveMenu {
  items = $state<MenuItem[]>([]);

  constructor(items: MenuItem[]) {
    this.items = items;
  }

  build = (): Menu => ({ label: "Live actions", items: this.items });
}
