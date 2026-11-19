<script lang="ts">
  import type { DeviceCapabilities } from "../../lib/api/bindings/DeviceCapabilities";

  let { capabilities }: { capabilities: DeviceCapabilities } = $props();
</script>

{#if capabilities.modes.length === 0}
  <p class="hint">This device reports no modes.</p>
{:else}
  <ul aria-labelledby="hardware-title">
    {#each capabilities.modes as mode, index (index)}
      <li>
        <span>{mode.name}</span>
        {#if index === capabilities.active_mode}<span class="tag good">Active</span>{/if}
        {#if index === capabilities.direct_mode}<span class="tag">Direct Mode</span>{/if}
      </li>
    {/each}
  </ul>
  <p class="hint">
    {capabilities.active_mode === null ? "OpenRGB reports no active mode for this device." : "Active means active in the last device scan."}
    To run one, use Set lighting.
  </p>
{/if}

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    columns: 14rem auto;
    column-gap: var(--space-5);
  }

  li {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-3);
    align-items: baseline;
    padding: var(--space-1) 0;
    break-inside: avoid;
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
