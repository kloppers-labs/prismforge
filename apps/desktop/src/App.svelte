<script lang="ts">
  import { onMount } from "svelte";
  import ConnectionScreen from "./features/connection/ConnectionScreen.svelte";
  import DemoBanner from "./features/connection/DemoBanner.svelte";
  import SessionBar from "./features/connection/SessionBar.svelte";
  import WaitingPanel from "./features/connection/WaitingPanel.svelte";
  import { isSimulated, summarise } from "./features/connection/wording";
  import DiagnosticsDrawer from "./features/diagnostics/DiagnosticsDrawer.svelte";
  import Inventory from "./features/devices/Inventory.svelte";
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
  let diagnosticsButton: HTMLButtonElement | undefined = $state();

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let closed = false;
    app.attach().then((stop) => (closed ? stop() : (unlisten = stop)));
    return () => {
      closed = true;
      unlisten?.();
    };
  });

  function closeDiagnostics() {
    app.diagnosticsOpen = false;
    diagnosticsButton?.focus();
  }
</script>

<div class="app">
  <header class="top">
    <div class="brand">
      <Mark />
      <span class="name">PrismForge</span>
    </div>
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

  {#if app.problem}
    <div class="notice problem" role="alert">
      <ToneIcon tone="problem" />
      <p>{failureText(app.problem)}</p>
    </div>
  {/if}

  {#if session && !stopped && session.devices.length > 0 && summary && summary.reasons.length > 0}
    <div class="notice {summary.tone}" role="status">
      <ToneIcon tone={summary.tone} />
      <div>
        {#each summary.reasons as reason (reason)}
          <p>{reason}</p>
        {/each}
      </div>
    </div>
  {/if}

  <main>
    {#if !session || stopped}
      <ConnectionScreen {app} stopped={session} />
    {:else if session.devices.length === 0}
      <WaitingPanel {session} />
    {:else}
      <Inventory {app} {session} />
    {/if}
  </main>

  {#if app.diagnosticsOpen}
    <DiagnosticsDrawer {session} onclose={closeDiagnostics} />
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
