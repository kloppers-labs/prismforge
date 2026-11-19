<script lang="ts">
  import { onMount } from "svelte";
  import type { EffectInstance } from "../../lib/api/bindings/EffectInstance";
  import type { PreviewLimits } from "../../lib/api/bindings/PreviewLimits";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { nativeWindow } from "../../lib/window.svelte";
  import { ledCount } from "../devices/present";
  import { LiveWatcher } from "./live.svelte";
  import LiveRows from "./LiveRows.svelte";
  import { canStop, type StageRow } from "./present";
  import { PreviewPlayer } from "./preview.svelte";

  const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

  let { instance, rows, limits, session }: { instance: EffectInstance | null; rows: StageRow[]; limits: PreviewLimits; session: SessionSnapshot } = $props();

  let open = $state(true);
  let mode = $state<"draft" | "live">("draft");
  let documentShown = $state(true);
  const player = new PreviewPlayer(() => limits);
  const watcher = new LiveWatcher();
  let sending = $derived(rows.some((row) => canStop(session, row.target)));
  let demo = $derived(session.backend.origin === "Simulated");
  let shown = (leds: number) => Math.min(Math.max(leds, limits.leds.min), limits.leds.max);

  function onVisibility() {
    documentShown = document.visibilityState !== "hidden";
  }

  onMount(() => {
    player.open();
    void watcher.open();
    const motion = typeof window.matchMedia === "function" ? window.matchMedia(REDUCED_MOTION) : null;
    watcher.reduced = motion?.matches ?? false;
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      player.close();
      watcher.close();
      document.removeEventListener("visibilitychange", onVisibility);
    };
  });

  $effect(() => {
    player.show(open && mode === "draft" ? instance : null, rows.map((row) => row.leds));
  });

  $effect(() => {
    player.setWindowShown(nativeWindow.visible);
  });

  $effect(() => {
    const visible = open && mode === "live" && nativeWindow.visible && documentShown;
    watcher.watch(visible ? rows.map((row) => row.target) : []);
  });
</script>

<div class="stage" class:live={mode === "live"} class:draft={mode === "draft"}>
  <div class="head">
    <h4 id="preview-title">Preview</h4>
    <div class="states" role="group" aria-label="What the preview shows">
      <button type="button" class="small" aria-pressed={mode === "draft"} onclick={() => (mode = "draft")}>Draft preview</button>
      <button type="button" class="small" aria-pressed={mode === "live"} onclick={() => (mode = "live")}>Live output</button>
    </div>
    <button type="button" class="quiet" aria-expanded={open} aria-controls="preview-body" onclick={() => (open = !open)}>
      {open ? "Hide preview" : "Show preview"}
    </button>
  </div>
  <p class="badge" aria-live="polite">
    {#if mode === "draft"}
      <span class="mark" aria-hidden="true">◇</span><strong>Draft</strong> The pattern only. PrismForge sends nothing for it.
    {:else}
      <span class="mark" aria-hidden="true">●</span><strong>Live</strong>
      {demo ? "What PrismForge sends to the demo backend now. Nothing reaches hardware." : "What PrismForge sends now. PrismForge cannot see the device, so this is not a reading of the lights."}
    {/if}
  </p>
  <div id="preview-body" class="body">
    {#if open && mode === "draft"}
      {#each rows as row (row.key)}
        {@const frame = player.frameFor(row.leds)}
        <div class="row">
          {#if rows.length > 1}<p class="label">{row.label}</p>{/if}
          <div class="rail true-color" aria-hidden="true">
            {#if frame}
              {#each frame as color, index (index)}
                <span class="led" style:--led="rgb({color.r} {color.g} {color.b})"></span>
              {/each}
            {:else}
              {#each { length: shown(row.leds) }, index (index)}
                <span class="led off"></span>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
      <p class="caption">
        {rows.length === 1 ? `Preview of the pattern on ${ledCount(shown(rows[0]?.leds ?? 0))}. It is not what the device shows.` : "Preview of the pattern on each target's LEDs, on one clock, as the targets run it together. It is not what the devices show."}
        {#if player.reduced}Motion is reduced, so the preview shows one still frame.{/if}
      </p>
      {#if player.slowed}<p class="caption warn">Fast flashing is shown slowed on the screen. The devices get every frame.</p>{/if}
      {#if instance === null}
        <p class="caption">The preview waits for valid settings.</p>
      {:else if player.failed}
        <p class="caption">The preview could not be drawn for these settings.</p>
      {/if}
    {:else if open}
      {#if !sending}
        <p class="caption">No PrismForge effect runs on {rows.length === 1 ? "this target" : "these targets"}, so nothing is sent.</p>
      {/if}
      {#if watcher.refusal}
        <p class="caption warn" role="alert">The live output cannot be shown. {watcher.refusal}</p>
      {:else}
        <LiveRows {rows} output={watcher.output} />
      {/if}
      {#if watcher.reduced}<p class="caption">Motion is reduced, so the live view shows a new still frame every 2 seconds.</p>{/if}
      {#if watcher.slowed}<p class="caption warn">Fast flashing is shown slowed on the screen. The devices get every frame.</p>{/if}
    {/if}
  </div>
</div>

<style>
  .stage {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-m);
    border: 1px dashed var(--line-control);
  }

  .stage.live {
    border: 2px solid var(--ok);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .states {
    display: flex;
    gap: var(--space-1);
    margin-right: auto;
  }

  .states [aria-pressed="true"] {
    border-color: var(--accent);
    color: var(--text);
    font-weight: var(--weight-heading);
  }

  h4 {
    margin: 0;
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
  }

  .quiet {
    padding: var(--space-1) var(--space-2);
    font-size: var(--font-s);
  }

  .badge {
    display: flex;
    flex-wrap: wrap;
    gap: 0 var(--space-2);
    align-items: baseline;
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .badge strong {
    color: var(--text);
  }

  .live .badge .mark,
  .live .badge strong {
    color: var(--ok);
  }

  .body,
  .row {
    display: grid;
    gap: var(--space-2);
  }

  .label {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .rail {
    display: flex;
    gap: 3px;
    padding: var(--space-3);
    border-radius: var(--radius-m);
    background: linear-gradient(180deg, var(--surface-stage), var(--surface-stage-edge));
    box-shadow: inset 0 0 0 1px var(--surface-3);
  }

  .led {
    flex: 1 1 0;
    min-width: 3px;
    height: 18px;
    border-radius: 3px;
    background: var(--led);
    box-shadow:
      0 0 10px -1px var(--led),
      inset 0 0 0 1px var(--line-subtle);
  }

  .led.off {
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--surface-3);
  }

  .caption {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .warn {
    color: var(--warning);
  }

  @media (forced-colors: active) {
    .stage.live {
      border-color: Highlight;
    }

    .states [aria-pressed="true"] {
      border-color: Highlight;
    }
  }
</style>
