<script lang="ts">
  import { onMount } from "svelte";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { StoreView } from "../../lib/api/bindings/StoreView";
  import type { Names } from "../../lib/names";
  import { deviceKey, reference } from "../devices/present";
  import { errorName, eventText, raw, reportText, type ReportPart } from "./format";

  let { session, store, names, onclose }: { session: SessionSnapshot | null; store: StoreView | null; names: Names; onclose: () => void } = $props();

  let copied = $state<"done" | "failed" | null>(null);

  function nameOf(ref: string): string {
    const device = session?.devices.find((each) => reference(each) === ref);
    return device ? names.device(device) : ref;
  }

  function parts(): ReportPart[] {
    const list: ReportPart[] = [];
    if (store) {
      list.push({ title: "Profile store", rows: [["Store", store.kind]] });
    }
    if (!session) {
      return [...list, { title: "Connection", rows: [["Session", "none"]] }];
    }
    const last = session.diagnostics.last_error;
    list.push(
      {
        title: "Backend",
        rows: [
          ["Origin", session.backend.origin],
          ["Address", session.backend.address ?? "None"],
          ["Protocol version", String(session.backend.protocol_version ?? "Not negotiated")],
          ["Server name", session.backend.server_name ?? "Not reported"],
          ["Connection state", raw(session.connection)],
        ],
      },
      { title: "Last error", rows: last ? [["Operation", last.operation], ["At", `${last.at_ms} ms`], ["Error", raw(last.error)]] : [["Error", "none"]] },
      { title: "Counters", rows: counters.map(([label, value]) => [String(label), String(value)]) },
      {
        title: "Output",
        rows: [["Frame rate", String(session.output.frame_rate)], ...session.output.devices.map((output) => [output.device, `${raw(output.owner)} ${raw(output.condition)}`] as [string, string])],
      },
      { title: "Devices", rows: session.devices.map((device) => [device.descriptor.name, reference(device) ?? raw(device.identity)] as [string, string]) },
      { title: "Recent events", rows: events.map((event) => [`#${event.sequence} at ${event.at_ms} ms`, eventText(event.kind)] as [string, string]) },
    );
    return list;
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(reportText(parts()));
      copied = "done";
    } catch {
      copied = "failed";
    }
  }

  let heading: HTMLHeadingElement | undefined = $state();
  let root: HTMLElement | undefined = $state();
  let counters = $derived(
    session
      ? [
          ["Connect attempts", session.diagnostics.counters.connect_attempts],
          ["Failed connects", session.diagnostics.counters.connect_failures],
          ["Inventory refreshes", session.diagnostics.counters.inventory_refreshes],
          ["Failed refreshes", session.diagnostics.counters.refresh_failures],
          ["Commands refused as busy", session.diagnostics.counters.commands_refused_busy],
          ["Frames written", session.diagnostics.counters.frames_written],
          ["Ticks skipped", session.diagnostics.counters.ticks_skipped],
          ["Writes refused", session.diagnostics.counters.output_writes_refused],
          ["Writes that ended the connection", session.diagnostics.counters.output_writes_failed],
          ["Output wake-ups", session.diagnostics.counters.output_wakeups],
        ]
      : [],
  );
  let events = $derived(session ? [...session.diagnostics.events].reverse() : []);

  onMount(() => heading?.focus());

  function onkeydown(event: KeyboardEvent) {
    const inside = event.target instanceof Node && root?.contains(event.target);
    if (event.key === "Escape" && inside && !event.defaultPrevented && !document.querySelector("[aria-modal='true']")) {
      onclose();
    }
  }
</script>

<svelte:window {onkeydown} />

<aside id="diagnostics-drawer" class="drawer" aria-labelledby="diagnostics-title" bind:this={root}>
  <header>
    <h2 id="diagnostics-title" tabindex="-1" data-region-focus bind:this={heading}>Diagnostics</h2>
    <div class="tools">
      <button type="button" class="small" onclick={copy}>Copy as text</button>
      <button type="button" class="small" onclick={onclose}>Close</button>
    </div>
  </header>
  <p class="intro">What PrismForge knows about this session, for a bug report. Copy as text puts the server address, the device names and references, the counters, the last error and the recent events on the clipboard. The copy leaves out the folder path.</p>
  <p class="copied" role="status">
    {#if copied === "done"}Copied. Paste it into your report.{:else if copied === "failed"}PrismForge could not reach the clipboard. Select the values below and copy them.{/if}
  </p>

  {#if store}
    <section aria-labelledby="store-title">
      <h3 id="store-title">Profile store</h3>
      <dl>
        <dt>Store</dt>
        <dd>{store.kind === "Demo" ? "Demo profiles, kept apart from real ones" : "Profiles for real devices"}</dd>
        <dt>Folder</dt>
        <dd class="code">{store.folder}</dd>
      </dl>
    </section>
  {/if}

  {#if !session}
    <p class="empty">PrismForge has no connection now.</p>
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

    <section aria-labelledby="output-title">
      <h3 id="output-title">Output</h3>
      <dl>
        <dt>Frame rate</dt>
        <dd class="numeric">{session.output.frame_rate} per second</dd>
        <dt>Scene</dt>
        <dd class="code">{raw(session.output.scene.assignments)}</dd>
        {#each session.output.devices as output (output.device)}
          <dt>{nameOf(output.device)}<span class="ref code">{output.device}</span></dt>
          <dd class="code">{raw(output.owner)} {raw(output.condition)}</dd>
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
            <dt>{names.device(device)}</dt>
            <dd><span class="code">{reference(device) ?? `ambiguous: ${raw(device.identity)}`}</span>{#if names.device(device) !== device.descriptor.name}<span class="ref">Reported as <span>{device.descriptor.name}</span></span>{/if}</dd>
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
    flex: 1;
    min-width: 0;
    background: var(--surface-1);
    border-left: 1px solid var(--line-subtle);
    box-shadow: var(--shadow-3);
    overflow-y: auto;
    padding: var(--space-4) var(--space-5) var(--space-7);
    display: grid;
    gap: var(--space-5);
    align-content: start;
    animation: enter var(--dur-base) var(--ease-out);
  }

  @keyframes enter {
    from {
      transform: translateX(24px);
      opacity: 0;
    }
  }

  header {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
  }

  .tools {
    display: flex;
    gap: var(--space-2);
  }

  .intro {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .copied:empty {
    display: none;
  }

  .ref {
    display: block;
    color: var(--text-dim);
    font-size: var(--font-xs);
    font-weight: var(--weight-regular);
  }

  h2:focus {
    outline: none;
  }

  section {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--line-subtle);
  }

  h3 {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  dl {
    display: grid;
    grid-template-columns: minmax(0, 9rem) minmax(0, 1fr);
    gap: var(--space-1) var(--space-3);
    margin: 0;
  }

  dt {
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .code {
    font-family: var(--font-mono);
    font-size: var(--font-xs);
  }

  .plain {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-1);
  }

  .events li {
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }

  .events .numeric {
    color: var(--text-dim);
  }

  .empty {
    color: var(--text-dim);
  }
</style>
