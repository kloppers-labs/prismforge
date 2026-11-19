<script lang="ts">
  import { DEFAULT_HOST, DEFAULT_PORT } from "../../lib/api/client";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Notice from "../../lib/Notice.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import Prerequisites from "./Prerequisites.svelte";
  import { nextStep, summarise } from "./wording";

  let { app, stopped }: { app: AppState; stopped: SessionSnapshot | null } = $props();

  let host = $state(DEFAULT_HOST);
  let port = $state(DEFAULT_PORT);
  let edited = $state(false);
  let remembered = $derived(app.lifecycle.settings?.address ?? null);
  let portValid = $derived(Number.isInteger(port) && port >= 1 && port <= 65535);
  let hostValid = $derived(host.trim().length > 0);
  let fromLast = $derived(remembered !== null && remembered.host === host.trim() && remembered.port === port);
  let summary = $derived(stopped ? summarise(stopped) : null);
  let start = $derived(app.lifecycle.view?.start_profile ?? null);
  let marked = $derived(start !== null && typeof start === "object" && "Marked" in start ? start.Marked.name : null);
  let problem = $derived(
    !hostValid
      ? "Enter the address of the computer that runs OpenRGB."
      : !portValid
        ? "Enter a port from 1 to 65535. OpenRGB uses 6742 unless you changed it."
        : null,
  );

  $effect(() => {
    if (remembered && !edited) {
      host = remembered.host;
      port = remembered.port;
    }
  });

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
    <p>PrismForge controls RGB devices through OpenRGB, on this computer or on your network.</p>
  </header>

  {#if summary && summary.reasons.length > 0}
    <Notice tone="problem" role="alert" title="The last connection stopped.">
      {#each summary.reasons as reason (reason)}
        <p>{reason}</p>
      {/each}
      {#if stopped}<p>{nextStep(stopped)}</p>{/if}
    </Notice>
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
              oninput={() => (edited = true)}
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
              oninput={() => (edited = true)}
              class="numeric"
              aria-invalid={!portValid}
              aria-describedby="address-help"
            />
          </label>
        </div>
        <div id="address-help">
          {#if problem}
            <p class="problem" role="alert"><ToneIcon tone="problem" />{problem}</p>
          {:else if fromLast}
            <p class="help">The address of your last Connect.</p>
          {:else}
            <p class="help">OpenRGB on this computer uses 127.0.0.1 and port 6742.</p>
          {/if}
        </div>
        {#if marked}
          <p class="marked">Connecting applies {marked}, the profile marked to apply on connect, as soon as the devices are read.</p>
        {/if}
        <button class="primary" type="submit" disabled={!hostValid || !portValid || app.working !== null}>
          {#if app.working === "connect"}<span class="spinner" aria-hidden="true"></span>Connecting…{:else}Connect{/if}
        </button>
      </form>
      <Prerequisites />
    </section>

    <section class="choice demo" aria-labelledby="demo-title">
      <h2 id="demo-title">Explore demo mode</h2>
      <p>Try PrismForge with five simulated devices. Demo mode does not connect to OpenRGB and sends nothing to hardware.</p>
      <p>A striped “Demo mode” marker stays in the status bar while it runs.</p>
      <button type="button" onclick={() => app.startDemo()} disabled={app.working !== null}>
        {app.working === "demo" ? "Starting demo mode…" : "Start demo mode"}
      </button>
    </section>
  </div>
</div>

<style>
  .start {
    max-width: 920px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-5);
  }

  .intro {
    display: grid;
    gap: var(--space-2);
    max-width: 62ch;
  }

  .intro p {
    color: var(--text-dim);
  }

  .choices {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: var(--space-4);
    align-items: start;
  }

  .choice {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-5);
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
    gap: var(--space-3);
  }

  .fields {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 9ch;
    gap: var(--space-3);
  }

  label {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  label span {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  input {
    min-width: 0;
  }

  input[aria-invalid="true"] {
    border-color: var(--problem);
    border-width: 2px;
  }

  .help {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .marked {
    color: var(--text);
    font-size: var(--font-s);
  }

  .problem {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    font-size: var(--font-s);
    color: var(--problem);
  }

  .problem :global(svg) {
    margin-top: 2px;
  }

  form button {
    justify-self: start;
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border-radius: var(--radius-pill);
    border: 2px solid currentColor;
    border-right-color: transparent;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 760px) {
    .start {
      padding: var(--space-4);
    }

    .choices {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
