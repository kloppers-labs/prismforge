<script lang="ts">
  import Dialog from "../../lib/Dialog.svelte";
  import type { LifecycleState } from "./state.svelte";
  import { quitShortcut, trayName } from "./wording";

  let { lifecycle }: { lifecycle: LifecycleState } = $props();

  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
</script>

<Dialog title="Keep PrismForge running after you close the window?" {opener} onclose={() => lifecycle.answerClose(null)}>
  <p>If you keep it running, the window closes and PrismForge stays in {trayName()}. Your PrismForge effects keep running on your devices.</p>
  <p>To open the window again, use the PrismForge icon in {trayName()}. To quit completely, choose Quit PrismForge in that menu, on the Settings screen, or press {quitShortcut()}.</p>
  <p>If you quit, PrismForge stops its effects, puts back the lighting from before where it can, and ends.</p>
  <p>PrismForge remembers your choice. You can change it on the Settings screen.</p>
  {#snippet actions()}
    <button type="button" disabled={lifecycle.pending === "close"} onclick={() => lifecycle.answerClose(null)}>Cancel</button>
    <button type="button" disabled={lifecycle.pending === "close"} onclick={() => lifecycle.answerClose("Quit")}>Quit PrismForge</button>
    <button type="button" class="primary" disabled={lifecycle.pending === "close"} onclick={() => lifecycle.answerClose("Background")}>
      Keep running in {trayName()}
    </button>
  {/snippet}
</Dialog>
