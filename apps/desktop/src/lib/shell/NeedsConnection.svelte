<script lang="ts">
  import type { SessionSnapshot } from "../api/bindings/SessionSnapshot";
  import type { AppState } from "../state/app.svelte";
  import { capitalise, serverName, summarise } from "../../features/connection/wording";
  import { ended } from "./status";

  let {
    app,
    session,
    title,
    plural = true,
    onconnect,
  }: { app: AppState; session: SessionSnapshot | null; title: string; plural?: boolean; onconnect: () => void } = $props();

  let opens = $derived(plural ? "open" : "opens");

  let waiting = $derived(session !== null && !ended(session));
  let summary = $derived(session ? summarise(session) : null);
  let found = $derived(summary !== null && (summary.tone === "ok" || summary.tone === "warning"));
</script>

<section class="empty" aria-labelledby="needs-connection-title">
  <h1 id="needs-connection-title">{title}</h1>
  {#if waiting && session && summary}
    {#if found}
      <p>{capitalise(serverName(session))} reports no devices that PrismForge can use. {title} {opens} when it finds devices.</p>
    {:else}
      {#each summary.reasons as reason (reason)}
        <p>{reason}</p>
      {/each}
      <p>{title} {opens} when PrismForge is connected and has found devices.</p>
    {/if}
    <div class="actions">
      <button type="button" onclick={() => app.go("devices")}>Open Devices</button>
    </div>
  {:else}
    <p>{title} {plural ? "need" : "needs"} a connection. Connect to OpenRGB, or start demo mode to try PrismForge with simulated devices.</p>
    <div class="actions">
      <button type="button" class="primary" onclick={onconnect}>Connect to OpenRGB</button>
      <button type="button" disabled={app.working !== null} onclick={() => app.startDemo()}>Start demo mode</button>
    </div>
  {/if}
</section>

<style>
  .empty {
    max-width: 640px;
    margin: 0 auto;
    padding: var(--space-7) var(--space-6);
    display: grid;
    gap: var(--space-3);
  }

  p {
    color: var(--text-dim);
    max-width: 62ch;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
</style>
