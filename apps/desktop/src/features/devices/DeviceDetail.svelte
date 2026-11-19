<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { contextMenu } from "../../lib/shell/menu.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import { serverName } from "../connection/wording";
  import { controlSummary } from "../effects/present";
  import HardwareModeList from "./HardwareModeList.svelte";
  import IdentifyPanel from "./IdentifyPanel.svelte";
  import LedStrip from "./LedStrip.svelte";
  import { identifyReason, zoneMenu } from "./menu";
  import NameEditor from "./NameEditor.svelte";
  import { hasDirectMode, isAmbiguous, isLive, kindLabel, ledCount, reference, zoneShape, zoneSize } from "./present";

  let { app, session, device }: { app: AppState; session: SessionSnapshot; device: SessionDevice } = $props();

  let descriptor = $derived(device.descriptor);
  let capabilities = $derived(descriptor.capabilities);
  let ref = $derived(reference(device));
  let given = $derived(ref === null ? null : app.names.given(ref));
  let limit = $derived(ref === null ? null : (app.profiles.store?.name_limit ?? null));
  let naming = $derived(ref !== null && limit !== null);
  let blocked = $derived(identifyReason(app, session, device));
  let control = $derived(
    controlSummary(session, device, (id) => app.effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? null),
  );
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

  function nameDevice(name: string | null) {
    return ref === null ? Promise.resolve(null) : app.profiles.setDeviceName(ref, name);
  }

  function nameZone(zone: number) {
    return (name: string | null) => (ref === null ? Promise.resolve(null) : app.profiles.setZoneName(ref, zone, name));
  }

  function requested(zone: number | null): "rename" | "clear" | null {
    const asked = app.naming;
    return asked !== null && asked.device === ref && asked.zone === zone ? asked.action : null;
  }

  function identify(zone: number | null) {
    if (ref !== null && blocked === null) {
      app.identifyTarget({ device: ref, zone });
    }
  }
</script>

<article class="detail" aria-labelledby="device-title">
  <div class="inner">
    <header>
      <h2 id="device-title">{app.names.device(device)}</h2>
      {#if given !== null}
        <p class="sub">Reported as {descriptor.name}.</p>
      {/if}
      <p class="sub">{kindLabel(descriptor.kind)} by {descriptor.vendor || "an unknown vendor"}, {ledCount(descriptor.led_count)}</p>
      {#if control}<p class="control">{control}</p>{/if}
      <div class="actions">
        {#if ref !== null}
          {@const device = ref}
          <button type="button" class="primary" data-mirror="device.set-lighting" onclick={() => app.setLighting(device)}>Set lighting</button>
        {/if}
        <button
          type="button"
          data-mirror="device.identify"
          disabled={blocked !== null}
          aria-describedby={blocked ? "identify-blocked" : undefined}
          onclick={() => identify(null)}
        >
          Identify device
        </button>
        {#if naming && limit !== null}
          <NameEditor
            kind="device"
            subject={app.names.device(device)}
            {given}
            {limit}
            onsave={nameDevice}
            requested={requested(null)}
            onrequested={() => (app.naming = null)}
          />
        {/if}
      </div>
      <IdentifyPanel {app} {session} {device} {blocked} />
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
        <p>The devices report the same name, vendor and zones, and no serial number or location that differs. PrismForge has no way yet to tell them apart by hand, so Identify and effects are off for both.</p>
      </div>
    {/if}

    <section aria-labelledby="zones-title">
      <h3 id="zones-title">Zones</h3>
      {#if descriptor.zones.length === 0}
        <p class="hint">This device reports no zones.</p>
      {:else}
        <div class="table">
          <table>
            <thead>
              <tr>
                <th scope="col">Zone</th><th scope="col">LEDs</th><th scope="col">Shape</th>
                <th scope="col"><span class="visually-hidden">Actions</span></th>
              </tr>
            </thead>
            <tbody>
              {#each descriptor.zones as zone, index (index)}
                <tr use:contextMenu={() => zoneMenu(app, session, device, index)}>
                  <th scope="row">
                    {app.names.zone(device, index)}
                    {#if ref !== null && app.names.givenZone(ref, index) !== null}
                      <span class="reported">Reported as {zone.name}</span>
                    {/if}
                  </th>
                  <td class="numeric">{zone.led_count}</td>
                  <td>{zoneShape(zone)}<span class="reported">{zoneSize(zone)}</span></td>
                  <td><div class="row-actions">
                    <button
                      type="button"
                      class="small"
                      data-mirror="zone.identify"
                      disabled={blocked !== null}
                      aria-label="Identify zone {app.names.zone(device, index)}"
                      aria-describedby={blocked ? "identify-blocked" : undefined}
                      onclick={() => identify(index)}
                    >
                      Identify
                    </button>
                    {#if ref !== null && limit !== null}
                      <NameEditor
                        kind="zone"
                        subject={app.names.zone(device, index)}
                        given={app.names.givenZone(ref, index)}
                        {limit}
                        onsave={nameZone(index)}
                        requested={requested(index)}
                        onrequested={() => (app.naming = null)}
                      />
                    {/if}
                  </div></td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>

    <section aria-labelledby="colours-title">
      <h3 id="colours-title">Last reported colours</h3>
      <LedStrip colors={descriptor.colors} available={device.available} />
      <p class="hint">As {serverName(session)} reported them in the last device scan. Other software may have changed them since.</p>
    </section>

    <section aria-labelledby="hardware-title">
      <h3 id="hardware-title">Hardware modes</h3>
      <HardwareModeList {capabilities} />
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
        <dt>Device memory</dt>
        <dd>
          {capabilities.saves_to_device_memory
            ? "Some modes can be saved to the device. PrismForge never saves to device memory."
            : "No mode reports saving to the device."}
        </dd>
      </dl>
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
  </div>
</article>

<style>
  .detail {
    position: relative;
    min-width: 0;
    overflow-y: auto;
  }

  .inner {
    max-width: 880px;
    padding: var(--space-5) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-5);
    align-content: start;
  }

  header {
    display: grid;
    gap: var(--space-1);
  }

  .sub {
    color: var(--text-dim);
  }

  .control {
    color: var(--text);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin: var(--space-2) 0 var(--space-1);
  }

  section {
    display: grid;
    gap: var(--space-2);
  }

  h3 {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .note {
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-control);
    background: var(--surface-1);
    color: var(--text-dim);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  .note.warning {
    border-left-color: var(--warning);
    display: grid;
    gap: var(--space-1);
  }

  .note.warning p:first-child {
    color: var(--text);
    font-weight: var(--weight-strong);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .table {
    position: relative;
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th,
  td {
    text-align: left;
    padding: var(--space-2) var(--space-3) var(--space-2) 0;
    border-bottom: 1px solid var(--line-subtle);
    vertical-align: top;
  }

  thead th {
    color: var(--text-dim);
    font-weight: var(--weight-regular);
    font-size: var(--font-s);
  }

  tbody th {
    font-weight: var(--weight-strong);
  }

  .row-actions {
    display: flex;
    white-space: nowrap;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .reported {
    display: block;
    font-weight: var(--weight-regular);
    font-size: var(--font-xs);
    color: var(--text-dim);
  }

  .facts {
    display: grid;
    grid-template-columns: minmax(0, 9rem) minmax(0, 1fr);
    gap: var(--space-1) var(--space-4);
    margin: 0;
  }

  dt {
    color: var(--text-dim);
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
</style>
