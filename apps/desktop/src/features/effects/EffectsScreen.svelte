<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Details from "../../lib/Details.svelte";
  import { failureText } from "../../lib/errors";
  import Notice from "../../lib/Notice.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import { reference } from "../devices/present";
  import { closeText } from "../settings/wording";
  import EffectBar from "./EffectBar.svelte";
  import EffectEditor from "./EffectEditor.svelte";
  import EffectResults from "./EffectResults.svelte";
  import { serverName } from "../connection/wording";
  import ExternalControl from "./ExternalControl.svelte";
  import HandBack from "./HandBack.svelte";
  import HardwareModes from "./HardwareModes.svelte";
  import GroupsPanel from "./GroupsPanel.svelte";
  import { effectMenu, groupMenu, targetMenu, type EffectsMenuContext, type Reveal } from "./menu";
  import BatchBar from "./BatchBar.svelte";
  import BatchResults from "./BatchResults.svelte";
  import { applyBlocked, applyingText, declaredBy, deviceOf, targetLeds } from "./present";
  import TargetList from "./TargetList.svelte";
  import TargetStatus from "./TargetStatus.svelte";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let effects = $derived(app.effects);
  let closing = $derived(
    closeText({
      loaded: app.lifecycle.view !== null,
      tray: app.lifecycle.trayAvailable,
      onClose: app.lifecycle.settings?.on_close ?? null,
    }),
  );
  let target = $derived(effects.target);
  let device = $derived(target ? deviceOf(session, target) : null);
  let ref = $derived(device ? reference(device) : null);
  let catalog = $derived(effects.catalog);
  let held = $derived(applyingText(app.view));
  let blocked = $derived(device && target ? (held ?? applyBlocked(session, device, target.zone)) : null);
  let left = $derived(ref === null ? null : declaredBy(session, ref));
  let running = $derived(session.output.scene.assignments.length > 0);
  let firstLeds = $derived.by(() => {
    const first = effects.targets[0];
    const owner = first ? deviceOf(session, first) : null;
    return owner && first ? targetLeds(owner, first.zone) : 0;
  });
  let modesOpen = $state(false);
  let externalOpen = $state(false);
  let menus: EffectsMenuContext = $derived({ effects, groups: app.groups, session, names: app.names, held, reveal });

  $effect(() => {
    app.groups.follow(session);
  });

  async function reveal(part: Reveal) {
    await tick();
    if (part === "modes") {
      modesOpen = true;
    } else {
      externalOpen = true;
    }
    await tick();
    const control = part === "modes" ? document.querySelector<HTMLElement>("[data-mirror='target.modes']") : document.getElementById("program-name");
    control?.scrollIntoView?.({ block: "nearest" });
    control?.focus();
  }

  onMount(() => {
    effects.load();
    return () => effects.leave();
  });
</script>

<div class="two-pane">
  <TargetList
    {session}
    selection={effects.targets}
    shown={target}
    names={app.names}
    onselect={(next) => effects.select(next)}
    ontoggle={(next) => effects.toggle(next)}
    onrange={(next) => effects.extendTo(next)}
    onall={() => effects.selectAll()}
    onclear={() => effects.clear()}
    menu={(each, next) => targetMenu(menus, each, next)}
  >
    {#snippet groups()}
      <GroupsPanel groups={app.groups} {effects} {session} names={app.names} menu={(group) => groupMenu(menus, group)} />
    {/snippet}
    {#snippet foot()}
      <div class="stop-all">
        <button type="button" class="small" disabled={!running || held !== null || effects.working !== null} onclick={() => effects.stopAll()}>
          Stop all PrismForge effects
        </button>
        <p class="hint">{running ? "Stops every PrismForge effect, also on targets that are not selected." : "No PrismForge effect runs now."}</p>
      </div>
    {/snippet}
  </TargetList>
  <div class="workspace">
    <h1 class="visually-hidden">Effects</h1>
    {#if !catalog}
      <div class="inner">
        {#if effects.catalogProblem}
          <Notice tone="problem" role="alert" title="The effect list could not be loaded.">
            <p>{failureText(effects.catalogProblem)}</p>
            {#snippet actions()}
              <button type="button" class="small" onclick={() => effects.load()}>Load the effect list again</button>
            {/snippet}
          </Notice>
        {:else}
          <p class="empty" role="status">Loading the effect list.</p>
        {/if}
      </div>
    {:else if effects.several}
      <BatchBar {effects} {session} names={app.names} />
      <div class="scroll">
        <div class="inner">
          <BatchResults {effects} {session} names={app.names} />
          <EffectEditor {effects} {catalog} leds={firstLeds} locked={false} menu={(definition) => effectMenu(menus, definition, null, effects.blocked)} />
          <div class="how">
            <p>Runs while PrismForge runs and is connected. Every target runs the whole effect on its own LEDs, and all of them start together.</p>
            <Details summary="How effects run">
              <p>A PrismForge effect is computed by PrismForge, not by the device.</p>
              <p>{closing} When you stop an effect or quit, PrismForge asks {serverName(session)} to put back the earlier lighting.</p>
              <p>A target that cannot take the effect is left out, and the result says why. The other targets keep what they got.</p>
            </Details>
          </div>
        </div>
      </div>
    {:else if target && device}
      {#key `${target.device}-${target.zone}`}
        <EffectBar {effects} {session} {device} zone={target.zone} names={app.names} {blocked} {held} />
        <div class="scroll">
          <div class="inner">
            <BatchResults {effects} {session} names={app.names} />
            <EffectResults {effects} {session} {blocked} {held} />
            {#if left !== null && ref !== null}
              <HandBack {effects} device={ref} label={left} {held} />
            {:else}
              <TargetStatus {session} {device} zone={target.zone} />
            {/if}
            <EffectEditor
              {effects}
              {catalog}
              leds={targetLeds(device, target.zone)}
              locked={left !== null}
              menu={(definition) => effectMenu(menus, definition, device, blocked)}
            />
            <div class="how">
              <p>Runs while PrismForge runs and is connected.</p>
              <Details summary="How effects run">
                <p>A PrismForge effect is computed by PrismForge, not by the device.</p>
                <p>{closing} When you stop an effect or quit, PrismForge asks {serverName(session)} to put back the earlier lighting.</p>
                <p>PrismForge saves nothing by itself. To keep this lighting, save it as a profile on the Profiles screen. Profiles are saved on this computer, not on the device.</p>
                <p>After PrismForge starts again, no effect runs until you apply one, or until a profile marked to apply on connect is applied.</p>
              </Details>
            </div>
            <section class="other" aria-labelledby="other-title">
              <h3 id="other-title">Other ways to light this device</h3>
              <Details summary="Hardware modes, which run on the device itself" mirror="target.modes" bind:open={modesOpen}>
                <HardwareModes {effects} {session} {device} {held} />
              </Details>
              {#if left === null}
                <Details summary="Used by another program" bind:open={externalOpen}>
                  <ExternalControl {effects} {session} {device} {held} />
                </Details>
              {/if}
            </section>
          </div>
        </div>
      {/key}
    {:else if target}
      <div class="inner"><p class="empty">This device is not in the device list any more. Choose another target.</p></div>
    {:else}
      <div class="inner empty">
        <h2>Choose what to light</h2>
        <p>Choose a device or one of its zones on the left, or check several to light them together. Then choose an effect and apply it. Nothing changes on your devices until you apply.</p>
        <BatchResults {effects} {session} names={app.names} />
      </div>
    {/if}
  </div>
</div>

<style>
  .workspace {
    position: relative;
    min-width: 0;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    align-content: start;
    overflow: hidden;
  }

  .scroll {
    position: relative;
    min-height: 0;
    overflow-y: auto;
  }

  .inner {
    max-width: 920px;
    padding: var(--space-5) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-5);
    align-content: start;
  }

  .empty {
    display: grid;
    gap: var(--space-2);
    color: var(--text-dim);
  }

  .empty h2 {
    color: var(--text);
  }

  .stop-all {
    display: grid;
    gap: var(--space-1);
    justify-items: start;
    margin-top: auto;
    padding: var(--space-3) var(--space-2) 0;
    border-top: 1px solid var(--line-subtle);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .how {
    display: grid;
    gap: var(--space-1);
    color: var(--text-dim);
  }

  .other {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-4);
    border-top: 1px solid var(--line-subtle);
  }

  .other h3 {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .other :global(.details) {
    max-width: none;
  }

  @media (max-height: 520px) {
    .workspace {
      display: block;
      overflow-y: auto;
    }

    .scroll {
      overflow: visible;
    }
  }
</style>
