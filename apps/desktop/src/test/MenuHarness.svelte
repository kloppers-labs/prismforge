<script lang="ts">
  import ContextMenu from "../lib/shell/ContextMenu.svelte";
  import { contextMenu, type Menu } from "../lib/shell/menu.svelte";

  let { menu, build }: { menu?: Menu; build?: () => Menu } = $props();
  let open = () => build?.() ?? menu ?? null;
</script>

<button type="button">Before</button>
<button type="button" use:contextMenu={open}>Origin</button>
<input type="text" aria-label="Field" use:contextMenu={open} />
<div inert><button type="button" use:contextMenu={open}>Hidden origin</button></div>
<button type="button">Elsewhere</button>
<ContextMenu />
