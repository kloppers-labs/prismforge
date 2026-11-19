<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AppState } from "../../lib/state/app.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import Prerequisites from "./Prerequisites.svelte";
  import { capitalise, isSimulated, serverName, summarise } from "./wording";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let summary = $derived(summarise(session));
  let live = $derived(summary.tone === "ok" || summary.tone === "warning");
  let rejected = $derived(session.issues.length);
</script>

<section class="waiting" aria-labelledby="waiting-title">
  <div class="head">
    <ToneIcon tone={summary.tone} />
    <h1 id="waiting-title">
      {#if live && rejected > 0}
        No usable devices
      {:else if live}
        No devices found
      {:else if summary.retrying}
        Waiting for OpenRGB
      {:else}
        Connecting
      {/if}
    </h1>
  </div>
  {#each summary.reasons as reason (reason)}
    <p>{reason}</p>
  {/each}
  {#if live && rejected > 0}
    <p>
      PrismForge could not read the data of the {rejected === 1 ? "1 controller" : `${rejected} controllers`}
      {serverName(session)} reported. Diagnostics lists {rejected === 1 ? "it" : "them"} with the reason.
    </p>
  {:else if live}
    <p>{capitalise(serverName(session))} answered but reports no devices. Check the device list in {serverName(session)}, then rescan.</p>
  {:else if !summary.retrying}
    <p class="loading" aria-busy="true">Reading the device list when the server answers.</p>
  {/if}
  <div class="actions">
    {#if live}
      <button type="button" class="primary" disabled={app.working !== null} onclick={() => app.refresh()}>
        {app.working === "refresh" ? "Rescanning…" : "Rescan devices"}
      </button>
    {:else}
      <button type="button" disabled={app.working !== null} onclick={() => app.disconnect()}>Stop and change the address</button>
      {#if !isSimulated(session)}
        <button type="button" disabled={app.working !== null} onclick={() => app.startDemo()}>Start demo mode</button>
      {/if}
    {/if}
  </div>
  {#if !isSimulated(session) && !live}
    <Prerequisites />
  {/if}
</section>

<style>
  .waiting {
    max-width: 640px;
    margin: 0 auto;
    padding: var(--space-7) var(--space-6);
    display: grid;
    gap: var(--space-3);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  p {
    color: var(--text-dim);
    max-width: 62ch;
  }

  .loading::after {
    content: "";
    display: block;
    height: 2px;
    margin-top: var(--space-3);
    background: var(--spectrum);
    background-size: 200% 100%;
    animation: sweep 1.6s linear infinite;
    border-radius: 1px;
  }

  @keyframes sweep {
    from {
      background-position: 0 0;
    }
    to {
      background-position: 200% 0;
    }
  }
</style>
