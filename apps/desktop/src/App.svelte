<script lang="ts">
  import { onMount, tick } from "svelte";
  import ConnectionScreen from "./features/connection/ConnectionScreen.svelte";
  import WaitingPanel from "./features/connection/WaitingPanel.svelte";
  import DiagnosticsDrawer from "./features/diagnostics/DiagnosticsDrawer.svelte";
  import Inventory from "./features/devices/Inventory.svelte";
  import EffectsScreen from "./features/effects/EffectsScreen.svelte";
  import ProfilesScreen from "./features/profiles/ProfilesScreen.svelte";
  import ClosePrompt from "./features/settings/ClosePrompt.svelte";
  import SettingsScreen from "./features/settings/SettingsScreen.svelte";
  import { areaFor, cycleRegion, isQuit } from "./lib/shell/keyboard";
  import NeedsConnection from "./lib/shell/NeedsConnection.svelte";
  import NoticeSlot from "./lib/shell/NoticeSlot.svelte";
  import Rail from "./lib/shell/Rail.svelte";
  import { ended } from "./lib/shell/status";
  import StatusStrip from "./lib/shell/StatusStrip.svelte";
  import { AppState, type Screen } from "./lib/state/app.svelte";

  const app = new AppState();

  let session = $derived(app.session);
  let stopped = $derived(ended(session));
  let inventory = $derived(session !== null && !stopped && session.devices.length > 0);
  let store = $derived(app.profiles.store);
  let labels = $derived({
    names: app.names,
    session,
    effect: (id: string) => app.effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? null,
  });

  let rail: HTMLElement | undefined = $state();
  let strip: HTMLElement | undefined = $state();
  let notices: HTMLElement | undefined = $state();
  let main: HTMLElement | undefined = $state();
  let drawer: HTMLElement | undefined = $state();
  let diagnosticsButton: HTMLButtonElement | undefined = $state();
  let lastPageFocus: HTMLElement | null = null;

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let closed = false;
    app.attach().then((stop) => (closed ? stop() : (unlisten = stop)));
    return () => {
      closed = true;
      unlisten?.();
    };
  });

  function onkeydown(event: KeyboardEvent) {
    if (isQuit(event)) {
      event.preventDefault();
      app.lifecycle.quit();
      return;
    }
    if (event.key === "F6" && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      cycleRegion([rail, strip, notices, main, drawer], event.shiftKey);
      return;
    }
    const area = areaFor(event);
    if (area) {
      event.preventDefault();
      void goAndFocus(area.screen);
    }
  }

  async function goAndFocus(screen: Screen) {
    void app.go(screen);
    await tick();
    main?.focus();
  }

  async function connectForm() {
    void app.go("devices");
    await tick();
    document.getElementById("connect-host")?.focus();
  }

  async function revealPageFocus() {
    await tick();
    if (lastPageFocus?.isConnected) {
      lastPageFocus.scrollIntoView?.({ block: "nearest", inline: "nearest" });
    }
  }

  function toggleDiagnostics() {
    app.diagnosticsOpen = !app.diagnosticsOpen;
    void revealPageFocus();
  }

  function closeDiagnostics() {
    app.diagnosticsOpen = false;
    diagnosticsButton?.focus();
    void revealPageFocus();
  }

  function skipToContent() {
    main?.focus();
  }

  function trackPageFocus(event: FocusEvent) {
    if (event.target instanceof HTMLElement && event.target !== main) {
      lastPageFocus = event.target;
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="app">
  <button type="button" class="skip" onclick={skipToContent}>Skip to content</button>
  <Rail
    screen={app.screen}
    diagnosticsOpen={app.diagnosticsOpen}
    ongo={(screen) => app.go(screen)}
    ondiagnostics={toggleDiagnostics}
    bind:region={rail}
    bind:diagnosticsButton
  />

  <div class="content">
    <StatusStrip {app} onconnect={connectForm} bind:region={strip} />
    <div class="notices">
      <NoticeSlot {app} {labels} bind:region={notices} />
    </div>

    <main id="content" tabindex="-1" bind:this={main} onfocusin={trackPageFocus}>
      {#if app.screen === "settings"}
        <SettingsScreen lifecycle={app.lifecycle} onprofiles={() => app.openProfiles()} />
      {:else if app.screen === "profiles"}
        <ProfilesScreen {app} />
      {:else if app.screen === "effects"}
        {#if session && inventory}
          <EffectsScreen {app} {session} />
        {:else}
          <NeedsConnection {app} {session} title="Effects" onconnect={connectForm} />
        {/if}
      {:else if !session || stopped}
        <ConnectionScreen {app} stopped={session} />
      {:else if session.devices.length === 0}
        <WaitingPanel {session} />
      {:else}
        <Inventory {app} {session} />
      {/if}
    </main>

    {#if app.diagnosticsOpen}
      <div class="drawer" bind:this={drawer}>
        <DiagnosticsDrawer {session} {store} onclose={closeDiagnostics} />
      </div>
    {/if}
  </div>

  {#if app.lifecycle.view?.close_prompt}
    <ClosePrompt lifecycle={app.lifecycle} />
  {/if}
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
  }

  .skip {
    position: absolute;
    left: var(--space-2);
    top: var(--space-2);
    z-index: var(--z-popover);
    transform: translateY(-200%);
  }

  .skip:focus-visible {
    transform: none;
  }

  .content {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-rows: auto auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
  }

  .content > :global(.strip) {
    grid-column: 1 / -1;
    z-index: var(--z-sticky);
  }

  .notices {
    grid-column: 1;
    grid-row: 2;
    min-width: 0;
  }

  main {
    grid-column: 1;
    grid-row: 3;
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: auto;
  }

  main:focus-visible {
    outline-offset: -2px;
  }

  .drawer {
    grid-column: 2;
    grid-row: 2 / 4;
    min-height: 0;
    display: flex;
    width: clamp(300px, 34vw, 440px);
  }
</style>
