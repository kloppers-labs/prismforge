<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AcrossState } from "./across.svelte";

  let {
    across,
    session,
    reduced,
    slowed,
    note,
  }: { across: AcrossState; session: SessionSnapshot; reduced: boolean; slowed: boolean; note: string | null } = $props();

  let demo = $derived(session.backend.origin === "Simulated");
</script>

<div class="stage" class:live={across.stage === "live"} role="group" aria-label="Colours in the drawing">
  <div class="states">
    <button type="button" class="small" aria-pressed={across.stage === "draft"} onclick={() => (across.stage = "draft")}>Draft preview</button>
    <button type="button" class="small" aria-pressed={across.stage === "live"} onclick={() => (across.stage = "live")}>Live output</button>
  </div>
  <p class="badge" aria-live="polite">
    {#if across.stage === "draft"}
      <span class="mark" aria-hidden="true">◇</span><strong>Draft</strong> The pattern only. PrismForge sends nothing for it.
    {:else}
      <span class="mark" aria-hidden="true">●</span><strong>Live</strong>
      {demo ? "What PrismForge sends to the demo backend now. Nothing reaches hardware." : "What PrismForge sends now. PrismForge cannot see the device, so this is not a reading of the lights."}
    {/if}
  </p>
  {#if note}<p class="caption">{note}</p>{/if}
  {#if reduced}
    <p class="caption">{across.stage === "draft" ? "Motion is reduced, so the drawing shows one still frame." : "Motion is reduced, so the drawing shows a new still frame every 2 seconds."}</p>
  {/if}
  {#if slowed}<p class="caption warn">Fast flashing is shown slowed on the screen. The devices get every frame.</p>{/if}
</div>

<style>
  .stage {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border: 1px dashed var(--line-control);
    border-radius: var(--radius-m);
  }

  .stage.live {
    border: 2px solid var(--ok);
  }

  .states {
    display: flex;
    gap: var(--space-1);
  }

  .states [aria-pressed="true"] {
    border-color: var(--accent);
    font-weight: var(--weight-heading);
  }

  .badge,
  .caption {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .badge strong {
    color: var(--text);
    margin-right: var(--space-1);
  }

  .mark {
    margin-right: var(--space-1);
  }

  .live .badge .mark,
  .live .badge strong {
    color: var(--ok);
  }

  .warn {
    color: var(--warning);
  }

  @media (forced-colors: active) {
    .stage.live {
      border-color: Highlight;
    }
  }
</style>
