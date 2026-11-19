<script lang="ts">
  import { onMount } from "svelte";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { failureText } from "../../lib/errors";
  import type { AppState } from "../../lib/state/app.svelte";
  import EffectEditor from "./EffectEditor.svelte";
  import ExternalControl from "./ExternalControl.svelte";
  import FrameRate from "./FrameRate.svelte";
  import HardwareModes from "./HardwareModes.svelte";
  import { applyingText, deviceOf } from "./present";
  import TargetList from "./TargetList.svelte";
  import TargetStatus from "./TargetStatus.svelte";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let effects = $derived(app.effects);
  let target = $derived(effects.target);
  let device = $derived(target ? deviceOf(session, target) : null);
  let catalog = $derived(effects.catalog);
  let held = $derived(applyingText(app.view));

  onMount(() => {
    effects.load();
    return () => effects.leave();
  });
</script>

<div class="screen">
  <TargetList {session} selected={target} names={app.names} onselect={(next) => effects.select(next)} />
  <div class="workspace">
    {#if !catalog}
      {#if effects.catalogProblem}
        <div class="notice" role="alert">
          <p>The effect list could not be loaded. {failureText(effects.catalogProblem)}</p>
          <button type="button" onclick={() => effects.load()}>Load the effect list again</button>
        </div>
      {:else}
        <p class="empty" role="status">Loading the effect list.</p>
      {/if}
    {:else}
      {#if target && device}
        {#key `${target.device}-${target.zone}`}
          <TargetStatus {session} {device} zone={target.zone} names={app.names} />
          <EffectEditor {effects} {catalog} {session} {device} zone={target.zone} {held} />
          <HardwareModes {effects} {session} {device} {held} />
          <ExternalControl {effects} {session} {device} {held} />
        {/key}
      {:else if target}
        <p class="empty">This device is not in the device list any more. Choose another target.</p>
      {:else}
        <div class="empty">
          <h2>Effects</h2>
          <p>Choose a device or one of its zones on the left. Then choose an effect and apply it.</p>
        </div>
      {/if}
      <FrameRate {effects} range={catalog.frame_rate} current={session.output.frame_rate} {held} />
    {/if}
  </div>
</div>

<style>
  .screen {
    display: grid;
    grid-template-columns: minmax(260px, 320px) 1fr;
    min-height: 0;
    height: 100%;
  }

  .workspace {
    overflow-y: auto;
    padding: 28px 36px 48px;
    display: grid;
    gap: 32px;
    align-content: start;
    max-width: 920px;
  }

  .workspace > :global(section) {
    padding-top: 24px;
    border-top: 1px solid var(--ink-3);
  }

  .workspace > :global(.editor) {
    border-top: 0;
    padding-top: 0;
  }

  .empty {
    display: grid;
    gap: 6px;
    color: var(--text-dim);
  }

  .notice {
    display: grid;
    gap: 8px;
    justify-items: start;
  }
</style>
