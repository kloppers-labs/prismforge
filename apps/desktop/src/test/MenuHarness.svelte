<script lang="ts">
  import ContextMenu from "../lib/shell/ContextMenu.svelte";
  import { contextMenu, type Menu } from "../lib/shell/menu.svelte";

  let { menu }: { menu: Menu } = $props();
</script>

<button type="button">Before</button>
<button type="button" use:contextMenu={() => menu}>Origin</button>
<input type="text" aria-label="Field" use:contextMenu={() => menu} />
<div inert><button type="button" use:contextMenu={() => menu}>Hidden origin</button></div>
<button type="button">Elsewhere</button>
<ContextMenu />
