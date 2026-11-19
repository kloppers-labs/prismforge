<script lang="ts">
  import Switch from "../../lib/Switch.svelte";

  let { on, status, onchange }: { on: boolean; status: string | null; onchange: (on: boolean) => void } = $props();

  const id = `on-connect-${Math.random().toString(36).slice(2)}`;
</script>

<div class="switch-row">
  <Switch label="Apply this profile when PrismForge connects" {on} describedby={id} {onchange} />
  <p {id} class="hint">When PrismForge connects, it applies this profile once. PrismForge connects by itself only when “Connect to OpenRGB when PrismForge starts” is on in Settings.</p>
  <p class="hint">Only one profile can do this; turning it on here turns it off for the others.</p>
  {#if status}
    <p class="status" role="status" aria-label="Apply on connect">{status}</p>
  {/if}
</div>

<style>
  .switch-row {
    display: grid;
    gap: var(--space-1);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
    max-width: 72ch;
  }

  .status {
    max-width: 72ch;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-control);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
