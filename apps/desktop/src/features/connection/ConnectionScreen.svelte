<script lang="ts">
  import { DEFAULT_HOST, DEFAULT_PORT } from "../../lib/api/client";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AppState } from "../../lib/state/app.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import Prerequisites from "./Prerequisites.svelte";
  import { summarise } from "./wording";

  let { app, stopped }: { app: AppState; stopped: SessionSnapshot | null } = $props();

  let host = $state(DEFAULT_HOST);
  let port = $state(DEFAULT_PORT);
  let portValid = $derived(Number.isInteger(port) && port >= 1 && port <= 65535);
  let hostValid = $derived(host.trim().length > 0);
  let summary = $derived(stopped ? summarise(stopped) : null);

  function connect(event: SubmitEvent) {
    event.preventDefault();
    if (hostValid && portValid) {
      app.connect(host.trim(), port);
    }
  }
</script>

<div class="start">
  <header class="intro">
    <h1>Connect PrismForge to your lighting</h1>
    <p>PrismForge controls RGB devices through OpenRGB, which must already run on this computer or on your network.</p>
  </header>

  {#if summary && summary.reasons.length > 0}
    <div class="stopped" role="alert">
      <ToneIcon tone="problem" />
      <div>
        <p class="headline">The last connection stopped.</p>
        {#each summary.reasons as reason (reason)}
          <p>{reason}</p>
        {/each}
      </div>
    </div>
  {/if}

  <div class="choices">
    <section class="choice" aria-labelledby="openrgb-title">
      <h2 id="openrgb-title">Connect to OpenRGB</h2>
      <form onsubmit={connect} novalidate>
        <div class="fields">
          <label class="host">
            <span>Host</span>
            <input
              id="connect-host"
              bind:value={host}
              autocomplete="off"
              spellcheck="false"
              aria-invalid={!hostValid}
              aria-describedby="address-help"
            />
          </label>
          <label class="port">
            <span>Port</span>
            <input
              type="number"
              min="1"
              max="65535"
              bind:value={port}
              class="numeric"
              aria-invalid={!portValid}
              aria-describedby="address-help"
            />
          </label>
        </div>
        <p id="address-help" class="help">
          {#if !hostValid}
            Enter the address of the computer that runs OpenRGB.
          {:else if !portValid}
            Enter a port from 1 to 65535. OpenRGB uses 6742 unless you changed it.
          {:else}
            OpenRGB on this computer uses 127.0.0.1 and port 6742.
          {/if}
        </p>
        <button class="primary" type="submit" disabled={!hostValid || !portValid || app.working !== null}>
          {app.working === "connect" ? "Connecting" : "Connect"}
        </button>
      </form>
      <Prerequisites />
    </section>

    <section class="choice demo" aria-labelledby="demo-title">
      <h2 id="demo-title">Explore demo mode</h2>
      <p>Try PrismForge with five simulated devices. Demo mode does not connect to OpenRGB and sends nothing to hardware.</p>
      <p>While demo mode runs, a striped “Demo mode” marker stays in the status bar at the top of every screen.</p>
      <button type="button" onclick={() => app.startDemo()} disabled={app.working !== null}>
        {app.working === "demo" ? "Starting demo mode" : "Start demo mode"}
      </button>
    </section>
  </div>
</div>

<style>
  .start {
    max-width: 920px;
    margin: 0 auto;
    padding: 48px 32px;
    display: grid;
    gap: 28px;
  }

  .intro {
    display: grid;
    gap: 8px;
    max-width: 62ch;
  }

  .intro p {
    color: var(--text-dim);
    font-size: 15px;
  }

  .stopped {
    display: flex;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--problem-line);
    border-radius: var(--radius-m);
    background: var(--problem-surface);
  }

  .stopped div {
    display: grid;
    gap: 4px;
  }

  .headline {
    font-weight: var(--weight-strong);
  }

  .choices {
    display: grid;
    grid-template-columns: 3fr 2fr;
    gap: 20px;
    align-items: start;
  }

  .choice {
    display: grid;
    gap: 16px;
    padding: 24px;
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
  }

  .choice p {
    color: var(--text-dim);
  }

  .demo {
    border-style: dashed;
    border-color: var(--warning-line);
  }

  .demo button {
    justify-self: start;
  }

  form {
    display: grid;
    gap: 10px;
  }

  .fields {
    display: grid;
    grid-template-columns: 1fr 120px;
    gap: 12px;
  }

  label {
    display: grid;
    gap: 4px;
  }

  label span {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  input[aria-invalid="true"] {
    border-color: var(--problem);
  }

  .help {
    font-size: var(--font-s);
    color: var(--text-faint);
  }

  form button {
    justify-self: start;
  }

  @media (max-width: 980px) {
    .choices {
      grid-template-columns: 1fr;
    }
  }
</style>
