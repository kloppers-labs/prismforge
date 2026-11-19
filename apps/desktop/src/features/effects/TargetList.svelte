<script lang="ts">
  import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { deviceKey, hasDirectMode, ledCount, reference } from "../devices/present";
  import { shortOwner } from "./present";

  let {
    session,
    selected,
    names,
    onselect,
  }: {
    session: SessionSnapshot;
    selected: OutputTarget | null;
    names: Names;
    onselect: (target: OutputTarget) => void;
  } = $props();

  let entries = $derived(
    session.devices.map((device, index) => ({ key: deviceKey(device, index), ref: reference(device), device })),
  );

  function current(ref: string | null, zone: number | null): "true" | undefined {
    return selected !== null && ref === selected.device && zone === selected.zone ? "true" : undefined;
  }
</script>

<nav class="targets" aria-labelledby="targets-title">
  <h2 id="targets-title" class="title">Targets</h2>
  <ul>
    {#each entries as { key, ref, device } (key)}
      <li>
        <div class="device" role="group" aria-labelledby="target-{key}">
          <p id="target-{key}" class="name">{names.device(device)}</p>
          <p class="owner">{shortOwner(session, device)}</p>
          {#if !device.available || !hasDirectMode(device)}
            <p class="tags">
              {#if !device.available}<span class="tag unavailable">Unavailable</span>{/if}
              {#if !hasDirectMode(device)}<span class="tag">No Direct Mode</span>{/if}
            </p>
          {/if}
          {#if ref === null}
            <p class="reason">PrismForge cannot tell this device apart from an identical one, so it cannot be a target.</p>
          {/if}
          <ul class="parts">
            <li>
              <button type="button" disabled={ref === null} aria-current={current(ref, null)} onclick={() => ref !== null && onselect({ device: ref, zone: null })}>
                Whole device <span class="meta">{ledCount(device.descriptor.led_count)}</span>
              </button>
            </li>
            {#if device.descriptor.zones.length > 1}
              {#each device.descriptor.zones as zone, index (index)}
                <li>
                  <button type="button" disabled={ref === null} aria-current={current(ref, index)} onclick={() => ref !== null && onselect({ device: ref, zone: index })}>
                    {names.zone(device, index)} <span class="meta">{ledCount(zone.led_count)}</span>
                  </button>
                </li>
              {/each}
            {/if}
          </ul>
        </div>
      </li>
    {/each}
  </ul>
</nav>

<style>
  .targets {
    border-right: 1px solid var(--line-subtle);
    background: var(--surface-1);
    overflow-y: auto;
    padding: 16px 12px;
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
    padding: 0 8px 10px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .targets > ul {
    display: grid;
    gap: 14px;
  }

  .device {
    display: grid;
    gap: 2px;
    padding: 0 8px;
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .owner,
  .reason {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .tags {
    display: flex;
    gap: 4px;
    margin: 2px 0;
  }

  .tag {
    font-size: var(--font-xs);
    padding: 0 6px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--line-subtle);
    color: var(--text-dim);
  }

  .unavailable {
    border-style: dashed;
  }

  .parts {
    display: grid;
    gap: 2px;
    margin-top: 4px;
  }

  .parts button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    padding: 5px 10px 5px 14px;
    position: relative;
    font-weight: var(--weight-regular);
  }

  .parts button:hover:not(:disabled) {
    background: var(--surface-2);
  }

  .parts button[aria-current="true"] {
    background: var(--surface-2);
    border-color: var(--line-subtle);
  }

  .parts button[aria-current="true"]::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 7px;
    bottom: 7px;
    width: 3px;
    border-radius: 2px;
    background: var(--spectrum);
    background-size: 100% 600%;
  }

  .meta {
    color: var(--text-faint);
    font-size: var(--font-s);
  }
</style>
