<script lang="ts">
  import { onMount } from "svelte";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { deviceKey, reference } from "../devices/present";
  import { errorName, eventText, raw } from "./format";

  let { session, onclose }: { session: SessionSnapshot | null; onclose: () => void } = $props();

  let heading: HTMLHeadingElement | undefined = $state();
  let counters = $derived(
    session
      ? [
          ["Connect attempts", session.diagnostics.counters.connect_attempts],
          ["Failed connects", session.diagnostics.counters.connect_failures],
          ["Inventory refreshes", session.diagnostics.counters.inventory_refreshes],
          ["Failed refreshes", session.diagnostics.counters.refresh_failures],
          ["Commands refused as busy", session.diagnostics.counters.commands_refused_busy],
        ]
      : [],
  );
  let events = $derived(session ? [...session.diagnostics.events].reverse() : []);

  onMount(() => heading?.focus());

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onclose();
    }
  }
</script>

<svelte:window {onkeydown} />

<aside class="drawer" aria-labelledby="diagnostics-title">
  <header>
    <h2 id="diagnostics-title" tabindex="-1" bind:this={heading}>Diagnostics</h2>
    <button type="button" onclick={onclose}>Close</button>
  </header>

  {#if !session}
    <p class="empty">No connection has been started.</p>
  {:else}
    <section aria-labelledby="backend-title">
      <h3 id="backend-title">Backend</h3>
      <dl>
        <dt>Origin</dt>
        <dd>{session.backend.origin === "Simulated" ? "Simulated (demo mode)" : "OpenRGB SDK server"}</dd>
        <dt>Address</dt>
        <dd>{session.backend.address ?? "None"}</dd>
        <dt>Protocol version</dt>
        <dd class="numeric">{session.backend.protocol_version ?? "Not negotiated"}</dd>
        <dt>Server name</dt>
        <dd>{session.backend.server_name ?? "Not reported"}</dd>
        <dt>Connection state</dt>
        <dd class="code">{raw(session.connection)}</dd>
        <dt>Identify running</dt>
        <dd class="code">{session.identify ? raw(session.identify) : "No"}</dd>
      </dl>
    </section>

    <section aria-labelledby="error-title">
      <h3 id="error-title">Last error</h3>
      {#if session.diagnostics.last_error}
        {@const last = session.diagnostics.last_error}
        <dl>
          <dt>Operation</dt>
          <dd>{last.operation}</dd>
          <dt>At</dt>
          <dd class="numeric">{last.at_ms} ms after the session started</dd>
          <dt>Error</dt>
          <dd>{errorName(last.error)}</dd>
          <dt>Raw error</dt>
          <dd class="code">{raw(last.error)}</dd>
          <dt>Ended the connection</dt>
          <dd class="code">{last.ended_connection ? raw(last.ended_connection) : "No"}</dd>
        </dl>
      {:else}
        <p class="empty">No error in this session.</p>
      {/if}
    </section>

    <section aria-labelledby="counters-title">
      <h3 id="counters-title">Counters</h3>
      <dl>
        {#each counters as [label, value] (label)}
          <dt>{label}</dt>
          <dd class="numeric">{value}</dd>
        {/each}
      </dl>
    </section>

    <section aria-labelledby="ids-title">
      <h3 id="ids-title">Device references</h3>
      {#if session.devices.length === 0}
        <p class="empty">No devices.</p>
      {:else}
        <dl>
          {#each session.devices as device, index (deviceKey(device, index))}
            <dt>{device.descriptor.name}</dt>
            <dd class="code">{reference(device) ?? `ambiguous: ${raw(device.identity)}`}</dd>
          {/each}
        </dl>
      {/if}
      {#if session.issues.length > 0}
        <h3 id="issues-title">Rejected controllers</h3>
        <ul class="plain" aria-labelledby="issues-title">
          {#each session.issues as issue (issue.runtime_id)}
            <li class="code">Controller ID {issue.runtime_id}: {issue.reason}. {issue.detail}</li>
          {/each}
        </ul>
      {/if}
    </section>

    <section aria-labelledby="events-title">
      <h3 id="events-title">Recent events, newest first</h3>
      {#if events.length === 0}
        <p class="empty">No events yet.</p>
      {:else}
        <ol class="plain events">
          {#each events as event (event.sequence)}
            <li class="code"><span class="numeric">#{event.sequence} at {event.at_ms} ms</span> {eventText(event.kind)}</li>
          {/each}
        </ol>
      {/if}
    </section>
  {/if}
</aside>

<style>
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(480px, 92vw);
    background: var(--ink-1);
    border-left: 1px solid var(--line);
    box-shadow: -18px 0 40px rgb(0 0 0 / 0.35);
    overflow-y: auto;
    padding: 20px 24px 40px;
    display: grid;
    gap: 24px;
    align-content: start;
    z-index: 10;
    animation: enter 160ms ease-out;
  }

  @keyframes enter {
    from {
      transform: translateX(24px);
      opacity: 0;
    }
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h2:focus {
    outline: none;
  }

  section {
    display: grid;
    gap: 8px;
  }

  dl {
    display: grid;
    grid-template-columns: 150px 1fr;
    gap: 4px 12px;
    margin: 0;
  }

  dt {
    color: var(--text-faint);
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .code {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 12px;
  }

  .plain {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }

  .events li {
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }

  .events .numeric {
    color: var(--text-faint);
  }

  .empty {
    color: var(--text-faint);
  }
</style>
