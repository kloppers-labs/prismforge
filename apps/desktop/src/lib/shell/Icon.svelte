<script lang="ts" module>
  export type IconName = "devices" | "effects" | "profiles" | "diagnostics" | "settings" | "chevron";
</script>

<script lang="ts">
  let { name }: { name: IconName } = $props();
</script>

<svg class="icon" viewBox="0 0 20 20" aria-hidden="true" focusable="false">
  {#if name === "devices"}
    <rect x="2.8" y="5" width="14.4" height="9.4" rx="2" />
    <path d="M6 14.4 V16.6 M10 14.4 V16.6 M14 14.4 V16.6" />
    <circle cx="6.4" cy="9.7" r="1.1" />
    <circle cx="10" cy="9.7" r="1.1" />
    <circle cx="13.6" cy="9.7" r="1.1" />
  {:else if name === "effects"}
    <path d="M2.4 16.4 L8 3.6 L13.6 16.4 Z" />
    <path d="M1 10.8 L5.4 10.2" />
    <path d="M11.6 9.4 L18.6 5.8 M12.8 12.6 L18.8 14.2" />
  {:else if name === "profiles"}
    <path d="M10 2.6 L16.4 6.3 L16.4 13.7 L10 17.4 L3.6 13.7 L3.6 6.3 Z" />
    <path d="M7 8.6 H13 M7 11.4 H11.4" />
  {:else if name === "diagnostics"}
    <rect x="2.6" y="3.6" width="14.8" height="12.8" rx="2.4" />
    <path d="M5 10.6 H7.4 L9 7 L11 13.6 L12.6 10.6 H15" />
  {:else if name === "settings"}
    <path d="M3.4 6 H16.6 M3.4 10 H16.6 M3.4 14 H16.6" />
    <circle cx="7.4" cy="6" r="1.7" class="knob" />
    <circle cx="12.8" cy="10" r="1.7" class="knob" />
    <circle cx="9" cy="14" r="1.7" class="knob" />
  {:else}
    <path d="M6 8 L10 12 L14 8" />
  {/if}
</svg>

<style>
  .icon {
    width: 20px;
    height: 20px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .knob {
    fill: var(--icon-knob, var(--surface-1));
  }
</style>
