<script lang="ts">
  import type { AppState } from "../state/app.svelte";
  import ConnectionChip from "./ConnectionChip.svelte";
  import DemoMarker from "./DemoMarker.svelte";
  import ProfileChip from "./ProfileChip.svelte";
  import { connectionName, connectionStatus, profileStatus } from "./status";

  let { app, onconnect, region = $bindable() }: { app: AppState; onconnect: () => void; region?: HTMLElement } = $props();

  let connection = $derived(connectionStatus(app.session));
  let profile = $derived(profileStatus(app.view, app.profiles.store));
  let background = $derived(
    app.lifecycle.settings?.on_close === "Background" && app.lifecycle.trayAvailable
      ? "Closing the window keeps PrismForge running in the background."
      : null,
  );
  let announcement = $derived(
    [connectionName(connection), profile?.name, connection.demo ? "Demo mode: simulated devices" : null].filter(Boolean).join(". "),
  );
</script>

<section class="strip" class:demo={connection.demo} aria-label="Status" bind:this={region}>
  <ConnectionChip {app} status={connection} {background} {onconnect} />
  {#if connection.demo}
    <DemoMarker />
  {/if}
  {#if profile}
    <span class="profile">
      <span class="divider" aria-hidden="true"></span>
      <ProfileChip {app} status={profile} />
    </span>
  {/if}
  <p class="visually-hidden" role="status" aria-label="Status summary">{announcement}</p>
</section>

<style>
  .strip {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-2);
    min-height: 48px;
    padding: var(--space-1) var(--space-4) var(--space-1) var(--space-3);
    background: var(--surface-1);
    border-bottom: 1px solid var(--line-subtle);
  }

  .strip::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: -1px;
    height: 1px;
    background: var(--spectrum);
    opacity: 0.55;
  }

  .strip.demo {
    padding-bottom: var(--space-2);
  }

  .strip.demo::after {
    height: 3px;
    background: var(--demo-stripes);
    opacity: 1;
  }

  .profile {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    max-width: 100%;
  }

  .divider {
    width: 1px;
    height: 20px;
    background: var(--line-subtle);
  }
</style>
