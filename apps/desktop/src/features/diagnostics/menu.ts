import type { Menu } from "../../lib/shell/menu.svelte";

export function sectionMenu(title: string, copySection: () => unknown, copyAll: () => unknown): Menu {
  return {
    label: `Actions for ${title}`,
    items: [
      { id: "diagnostics.copy-section", label: "Copy section", run: copySection },
      { id: "diagnostics.copy-all", label: "Copy as text", run: copyAll },
    ],
  };
}
