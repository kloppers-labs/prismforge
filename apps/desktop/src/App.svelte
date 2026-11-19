<script lang="ts">
  import { onMount } from "svelte";
  import ConnectionScreen from "./features/connection/ConnectionScreen.svelte";
  import DemoBanner from "./features/connection/DemoBanner.svelte";
  import ProfileStatus from "./features/connection/ProfileStatus.svelte";
  import SessionBar from "./features/connection/SessionBar.svelte";
  import WaitingPanel from "./features/connection/WaitingPanel.svelte";
  import { isSimulated, summarise } from "./features/connection/wording";
  import DiagnosticsDrawer from "./features/diagnostics/DiagnosticsDrawer.svelte";
  import Inventory from "./features/devices/Inventory.svelte";
  import EffectsScreen from "./features/effects/EffectsScreen.svelte";
  import ConnectApplyNotice from "./features/profiles/ConnectApplyNotice.svelte";
  import ProfilesScreen from "./features/profiles/ProfilesScreen.svelte";
  import RepairedBanner from "./features/profiles/RepairedBanner.svelte";
  import ClosePrompt from "./features/settings/ClosePrompt.svelte";
  import SettingsScreen from "./features/settings/SettingsScreen.svelte";
  import TrayStopNotice from "./features/settings/TrayStopNotice.svelte";
  import { isMac } from "./features/settings/wording";
  import { failureText } from "./lib/errors";
  import Mark from "./lib/Mark.svelte";
  import { AppState } from "./lib/state/app.svelte";
  import ToneIcon from "./lib/ToneIcon.svelte";

  const app = new AppState();

  let session = $derived(app.session);
  let stopped = $derived(
    session !== null && typeof session.connection === "object" && "Disconnected" in session.connection
      ? session.connection.Disconnected.reason !== null
      : false,
  );
  let summary = $derived(session ? summarise(session) : null);
  let inventory = $derived(session !== null && !stopped && session.devices.length > 0);
  let diagnosticsButton: HTMLButtonElement | undefined = $state();
  let store = $derived(app.profiles.store);
  let active = $derived(app.profiles.active);
  let connectApply = $derived(app.connectApplyDismissed ? null : app.view.connect_apply);
  let labels = $derived({
    names: app.names,
    session,
    effect: (id: string) => app.effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? null,
  });

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
    if (!isMac() && event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey && event.key.toLowerCase() === "q") {
      event.preventDefault();
      app.lifecycle.quit();
    }
  }

  function closeDiagnostics() {
    app.diagnosticsOpen = false;
    diagnosticsButton?.focus();
  }
</script>

<svelte:window {onkeydown} />

<div class="app">
  <header class="top">
    <div class="brand">
      <Mark />
      <span class="name">PrismForge</span>
    </div>
    <nav class="screens" aria-label="Screens">
      <button type="button" aria-current={app.screen === "devices" ? "page" : undefined} onclick={() => (app.screen = "devices")}>
        {inventory ? "Devices" : "Connection"}
      </button>
      {#if inventory}
        <button type="button" aria-current={app.screen === "effects" ? "page" : undefined} onclick={() => (app.screen = "effects")}>
          Effects
        </button>
      {/if}
      <button type="button" aria-current={app.screen === "profiles" ? "page" : undefined} onclick={() => app.openProfiles()}>
        Profiles
      </button>
      <button type="button" aria-current={app.screen === "settings" ? "page" : undefined} onclick={() => (app.screen = "settings")}>
        Settings
      </button>
    </nav>
    {#if active && store}
      <ProfileStatus name={active.name} unsaved={store.summary.unsaved_changes} onopen={() => app.openProfiles()} />
    {/if}
    {#if session && !stopped}
      <SessionBar {app} {session} />
    {:else}
      <div class="spacer"></div>
    {/if}
    <button
      type="button"
      bind:this={diagnosticsButton}
      aria-expanded={app.diagnosticsOpen}
      onclick={() => (app.diagnosticsOpen = !app.diagnosticsOpen)}
    >
      Diagnostics
    </button>
  </header>

  {#if session && isSimulated(session)}
    <DemoBanner />
  {/if}

  {#if store && store.summary.notices.length > 0 && app.screen !== "profiles"}
    <RepairedBanner onopen={() => app.openProfiles()} />
  {/if}

  {#if app.lifecycle.trayStop}
    <TrayStopNotice
      report={app.lifecycle.trayStop}
      {session}
      label={(device, reported) => app.names.given(device) ?? reported}
      ondismiss={() => app.lifecycle.dismissStop()}
    />
  {/if}

  {#if connectApply}
    <ConnectApplyNotice applied={connectApply} {labels} ondismiss={() => (app.connectApplyDismissed = true)} />
  {/if}

  {#if app.problem}
    <div class="notice problem" role="alert">
      <ToneIcon tone="problem" />
      <p>{failureText(app.problem)}</p>
    </div>
  {/if}

  {#if session && !stopped && session.devices.length > 0 && summary && summary.reasons.length > 0}
    <div class="notice {summary.tone}" role="status" aria-label="Connection status">
      <ToneIcon tone={summary.tone} />
      <div>
        {#each summary.reasons as reason (reason)}
          <p>{reason}</p>
        {/each}
      </div>
    </div>
  {/if}

  <main>
    {#if app.screen === "settings"}
      <SettingsScreen lifecycle={app.lifecycle} onprofiles={() => app.openProfiles()} />
    {:else if app.screen === "profiles"}
      <ProfilesScreen {app} />
    {:else if !session || stopped}
      <ConnectionScreen {app} stopped={session} />
    {:else if session.devices.length === 0}
      <WaitingPanel {session} />
    {:else if app.screen === "effects"}
      <EffectsScreen {app} {session} />
    {:else}
      <Inventory {app} {session} />
    {/if}
  </main>

  {#if app.diagnosticsOpen}
    <DiagnosticsDrawer {session} {store} onclose={closeDiagnostics} />
  {/if}

  {#if app.lifecycle.view?.close_prompt}
    <ClosePrompt lifecycle={app.lifecycle} />
  {/if}
</div>

<style>
  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 10px 20px;
    border-bottom: 1px solid var(--line);
    background: var(--ink-1);
    position: relative;
  }

  .top::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: -1px;
    height: 1px;
    background: var(--spectrum);
    opacity: 0.55;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .name {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: 0.01em;
  }

  .screens {
    display: flex;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--line);
    border-radius: var(--radius-s);
    background: var(--ink-0);
  }

  .screens button {
    border: 0;
    background: transparent;
    padding: 4px 14px;
    color: var(--text-dim);
  }

  .screens button[aria-current="page"] {
    background: var(--ink-3);
    color: var(--text);
  }

  .spacer {
    flex: 1;
  }

  .notice {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 10px 20px;
    border-bottom: 1px solid var(--line);
    background: var(--ink-1);
  }

  .notice :global(svg) {
    margin-top: 3px;
  }

  .notice.problem {
    background: #2a1d1f;
  }

  .notice.warning {
    background: #2a2519;
  }

  main {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
</style>
