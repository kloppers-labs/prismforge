<script lang="ts">
  import type { Flow } from "../../lib/api/bindings/Flow";
  import type { ViewId } from "../../lib/api/bindings/ViewId";
  import { acrossText } from "./flow";

  let { view, flow }: { view: ViewId; flow: Flow } = $props();
</script>

<p class="across"><span aria-hidden="true">▦</span> {acrossText(view, flow)}. Changes keep this mode.</p>

<style>
  .across {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
