<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AppState } from "../../lib/state/app.svelte";
  import IdentifyPanel from "./IdentifyPanel.svelte";
  import LedStrip from "./LedStrip.svelte";
  import { hasDirectMode, isAmbiguous, isLive, kindLabel, ledCount, zoneShape, zoneSize } from "./present";

  let { app, session, device }: { app: AppState; session: SessionSnapshot; device: SessionDevice } = $props();

  let descriptor = $derived(device.descriptor);
  let capabilities = $derived(descriptor.capabilities);
  let details = $derived(
    (
      [
        ["Description", descriptor.description],
        ["Version", descriptor.version],
        ["Serial number", descriptor.serial],
        ["Location", descriptor.location],
      ] as const
    ).filter(([, value]) => value.trim().length > 0),
  );
</script>

<article class="detail" aria-labelledby="device-title">
  <header>
    <h2 id="device-title">{descriptor.name}</h2>
    <p class="sub">{kindLabel(descriptor.kind)} by {descriptor.vendor || "an unknown vendor"}</p>
  </header>

  {#if descriptor.origin === "Simulated"}
    <p class="note">Simulated device. There is no hardware behind it.</p>
  {/if}
  {#if !device.available}
    <p class="note">
      {isLive(session)
        ? "PrismForge saw this device earlier, but OpenRGB does not report it now. These are the last known details."
        : "PrismForge is not connected. These are the last known details."}
    </p>
  {/if}
  {#if isAmbiguous(device)}
    <div class="note warning">
      <p>This device looks identical to another device. PrismForge will not light it until it can tell the two apart.</p>
      <p>The devices report the same name, vendor and zones, and no serial number or location that differs. A way to tell them apart by hand comes in a later version, so Identify is off for both.</p>
    </div>
  {/if}

  <section aria-labelledby="colours-title">
    <h3 id="colours-title">Last reported colours</h3>
    <p class="hint">As OpenRGB reported them in the last device scan. Other software may have changed them since.</p>
    <LedStrip colors={descriptor.colors} />
  </section>

  <IdentifyPanel {app} {session} {device} />

  <section aria-labelledby="zones-title">
    <h3 id="zones-title">Zones</h3>
    {#if descriptor.zones.length === 0}
      <p class="hint">This device reports no zones.</p>
    {:else}
      <table>
        <thead>
          <tr><th scope="col">Zone</th><th scope="col">Shape</th><th scope="col">LEDs</th><th scope="col">Size</th></tr>
        </thead>
        <tbody>
          {#each descriptor.zones as zone, index (index)}
            <tr>
              <th scope="row">{zone.name}</th>
              <td>{zoneShape(zone)}</td>
              <td class="numeric">{zone.led_count}</td>
              <td>{zoneSize(zone)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  <section aria-labelledby="capabilities-title">
    <h3 id="capabilities-title">What PrismForge can do</h3>
    <dl class="facts">
      <dt>Direct Mode</dt>
      <dd>
        {hasDirectMode(device)
          ? "Yes. PrismForge can set the colour of each LED."
          : "No. PrismForge cannot set colours on this device. Only its hardware modes run."}
      </dd>
      <dt>LEDs</dt>
      <dd class="numeric">{ledCount(descriptor.led_count)}</dd>
      <dt>Device memory</dt>
      <dd>
        {capabilities.saves_to_device_memory
          ? "Some modes can be saved to the device. PrismForge never saves to device memory."
          : "No mode reports saving to the device."}
      </dd>
    </dl>
    <h3 class="modes-title" id="modes-title">Hardware modes</h3>
    {#if capabilities.modes.length === 0}
      <p class="hint">This device reports no modes.</p>
    {:else}
      <ul class="modes" aria-labelledby="modes-title">
        {#each capabilities.modes as mode, index (index)}
          <li>
            <span>{mode.name}</span>
            {#if index === capabilities.active_mode}<span class="tag">Active</span>{/if}
            {#if index === capabilities.direct_mode}<span class="tag">Direct Mode</span>{/if}
          </li>
        {/each}
      </ul>
      {#if capabilities.active_mode === null}
        <p class="hint">OpenRGB reports no active mode for this device.</p>
      {/if}
    {/if}
  </section>

  {#if details.length > 0}
    <section aria-labelledby="details-title">
      <h3 id="details-title">Details</h3>
      <dl class="facts">
        {#each details as [label, value] (label)}
          <dt>{label}</dt>
          <dd>{value}</dd>
        {/each}
      </dl>
    </section>
  {/if}
</article>

<style>
  .detail {
    overflow-y: auto;
    padding: 28px 36px 48px;
    display: grid;
    gap: 28px;
    align-content: start;
    max-width: 880px;
  }

  header {
    display: grid;
    gap: 4px;
  }

  .sub {
    color: var(--text-dim);
  }

  section {
    display: grid;
    gap: 10px;
  }

  .note {
    padding: 10px 14px;
    border-left: 3px solid var(--line);
    background: var(--ink-1);
    color: var(--text-dim);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  .note.warning {
    border-left-color: var(--warning);
    display: grid;
    gap: 6px;
  }

  .note.warning p:first-child {
    color: var(--text);
    font-weight: 600;
  }

  .hint {
    color: var(--text-faint);
    font-size: 13px;
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th,
  td {
    text-align: left;
    padding: 7px 12px 7px 0;
    border-bottom: 1px solid var(--ink-3);
  }

  thead th {
    color: var(--text-faint);
    font-weight: 500;
    font-size: 13px;
  }

  tbody th {
    font-weight: 550;
  }

  .facts {
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 6px 16px;
    margin: 0;
  }

  dt {
    color: var(--text-faint);
  }

  dd {
    margin: 0;
  }

  .modes-title {
    margin-top: 8px;
  }

  .modes {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .modes li {
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 4px 10px;
    border: 1px solid var(--line);
    border-radius: 999px;
  }

  .tag {
    font-size: 12px;
    color: var(--text-dim);
    border-left: 1px solid var(--line);
    padding-left: 6px;
  }
</style>
