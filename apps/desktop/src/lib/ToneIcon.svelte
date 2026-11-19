<script lang="ts">
  import type { Tone } from "../features/connection/wording";

  let { tone }: { tone: Tone | "idle" } = $props();
</script>

<svg class="tone {tone}" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
  {#if tone === "ok"}
    <circle cx="8" cy="8" r="6.5" />
    <path d="M5 8.2 L7.2 10.3 L11 6" />
  {:else if tone === "idle"}
    <circle cx="8" cy="8" r="6.5" />
    <path d="M5.5 8 L10.5 8" />
  {:else if tone === "waiting"}
    <circle cx="8" cy="8" r="6.5" />
    <path d="M8 4.6 L8 8 L10.4 9.6" />
  {:else if tone === "working"}
    <circle cx="8" cy="8" r="6.5" stroke-dasharray="3 2.2" />
  {:else if tone === "warning"}
    <path d="M8 2 L14.5 13.5 L1.5 13.5 Z" />
    <path d="M8 6.5 L8 9.5 M8 11.4 L8 11.6" />
  {:else}
    <circle cx="8" cy="8" r="6.5" />
    <path d="M5.6 5.6 L10.4 10.4 M10.4 5.6 L5.6 10.4" />
  {/if}
</svg>

<style>
  .tone {
    width: 16px;
    height: 16px;
    flex: none;
    fill: none;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .ok {
    stroke: var(--ok);
  }

  .working {
    stroke: var(--working);
  }

  .waiting {
    stroke: var(--working);
  }

  .warning {
    stroke: var(--warning);
  }

  .problem {
    stroke: var(--problem);
  }

  .idle {
    stroke: var(--text-dim);
  }
</style>
