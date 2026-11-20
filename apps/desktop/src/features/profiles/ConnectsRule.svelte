<script lang="ts">
  let { applies, status, onautomation }: { applies: boolean; status: string | null; onautomation: () => void } = $props();
</script>

<div class="rule">
  <p>
    {applies
      ? "The rule “When PrismForge connects” applies this profile. Rules are set in Automation."
      : "No rule applies this profile when PrismForge connects. Rules are set in Automation."}
  </p>
  <button type="button" class="small" onclick={onautomation}>Open Automation</button>
  {#if status}
    <p class="status" role="status" aria-label="Apply on connect">{status}</p>
  {/if}
</div>

<style>
  .rule {
    display: grid;
    gap: var(--space-1);
    justify-items: start;
    color: var(--text-dim);
    font-size: var(--font-s);
    max-width: 72ch;
  }

  .status {
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-control);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }
</style>
