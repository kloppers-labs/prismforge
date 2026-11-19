<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Tone } from "../features/connection/wording";
  import ToneIcon from "./ToneIcon.svelte";

  let {
    tone,
    role = "status",
    label,
    title,
    children,
    details,
    actions,
    ondismiss,
  }: {
    tone: Tone;
    role?: "status" | "alert";
    label?: string;
    title: string;
    children?: Snippet;
    details?: Snippet;
    actions?: Snippet;
    ondismiss?: () => void;
  } = $props();

  const id = `notice-${Math.random().toString(36).slice(2)}`;
  let open = $state(false);
</script>

<div class="notice {tone}" {role} aria-label={label}>
  <ToneIcon {tone} />
  <div class="body">
    <p class="title">{title}</p>
    {#if children}{@render children()}{/if}
    {#if details && open}
      <div id="{id}-details" class="details">{@render details()}</div>
    {/if}
  </div>
  <div class="actions">
    {#if details}
      <button type="button" class="small quiet" aria-expanded={open} aria-controls="{id}-details" onclick={() => (open = !open)}>
        {open ? "Hide details" : "Details"}
      </button>
    {/if}
    {#if actions}{@render actions()}{/if}
    {#if ondismiss}
      <button type="button" class="small" onclick={ondismiss}>Dismiss</button>
    {/if}
  </div>
</div>

<style>
  .notice {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: var(--space-2) var(--space-3);
    align-items: start;
    padding: var(--space-2) var(--space-4) var(--space-2) var(--space-3);
    border-left: 3px solid var(--tone, var(--line-control));
    background: var(--tone-surface, var(--surface-1));
  }

  .notice :global(svg) {
    margin-top: 3px;
  }

  .ok {
    --tone: var(--ok);
    --tone-surface: var(--ok-surface);
  }

  .working {
    --tone: var(--working);
  }

  .waiting {
    --tone: var(--working);
  }

  .warning {
    --tone: var(--warning);
    --tone-surface: var(--warning-surface);
  }

  .problem {
    --tone: var(--problem);
    --tone-surface: var(--problem-surface);
  }

  .body {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .title {
    font-weight: var(--weight-strong);
    overflow-wrap: anywhere;
  }

  .notice:focus-visible {
    outline-offset: -2px;
  }

  .details {
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  @media (max-width: 1099px) {
    .notice {
      grid-template-columns: auto minmax(0, 1fr);
    }

    .actions {
      grid-column: 2;
      justify-content: flex-start;
    }
  }
</style>
