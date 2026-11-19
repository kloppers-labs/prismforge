<script lang="ts">
  import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { contextMenu, type Menu } from "../../lib/shell/menu.svelte";
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import { deviceKey, hasDirectMode, ledCount, reference } from "../devices/present";
  import { targetMark } from "./present";

  let {
    session,
    selected,
    names,
    onselect,
    menu,
  }: {
    session: SessionSnapshot;
    selected: OutputTarget | null;
    names: Names;
    onselect: (target: OutputTarget) => void;
    menu: (device: SessionDevice, target: OutputTarget) => Menu | null;
  } = $props();

  function menuFor(device: SessionDevice, ref: string | null, zone: number | null): Menu | null {
    return ref === null ? null : menu(device, { device: ref, zone });
  }

  let list: HTMLElement | undefined = $state();
  let entries = $derived(
    session.devices.map((device, index) => ({ key: deviceKey(device, index), ref: reference(device), device, mark: targetMark(session, device) })),
  );
  let firstRef = $derived(entries.find((entry) => entry.ref !== null)?.ref ?? null);

  function current(ref: string | null, zone: number | null): "true" | undefined {
    return selected !== null && ref === selected.device && zone === selected.zone ? "true" : undefined;
  }

  let listed = $derived(
    selected !== null &&
      entries.some(
        (entry) => entry.ref === selected.device && (selected.zone === null || selected.zone < entry.device.descriptor.zones.length),
      ),
  );

  function stop(ref: string | null, zone: number | null): number {
    const anchor = listed ? selected : firstRef === null ? null : { device: firstRef, zone: null };
    return anchor !== null && ref === anchor.device && zone === anchor.zone ? 0 : -1;
  }

  function move(event: KeyboardEvent) {
    const rows = list ? [...list.querySelectorAll<HTMLButtonElement>("button.pick:not(:disabled)")] : [];
    const at = rows.indexOf(event.currentTarget as HTMLButtonElement);
    const next = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: rows.length - 1 }[event.key];
    const row = next === undefined ? undefined : rows[Math.min(Math.max(next, 0), rows.length - 1)];
    if (row) {
      event.preventDefault();
      row.click();
      row.focus();
    }
  }
</script>

<nav class="targets" aria-labelledby="targets-title" bind:this={list}>
  <h2 id="targets-title" class="title">Targets</h2>
  <ul>
    {#each entries as { key, ref, device, mark } (key)}
      <li>
        <div class="device" role="group" aria-labelledby="target-{key}" aria-describedby={ref === null ? `target-${key}-why` : undefined}>
          <p id="target-{key}" class="name">{names.device(device)}</p>
          {#if mark || !device.available || !hasDirectMode(device)}
            <p class="tags">
              {#if mark}<span class="tag {mark.tone}">{mark.text}</span>{/if}
              {#if !device.available}<span class="tag">Unavailable</span>{/if}
              {#if !hasDirectMode(device)}<span class="tag">No Direct Mode</span>{/if}
            </p>
          {/if}
          {#if ref === null}
            <p id="target-{key}-why" class="reason">PrismForge cannot tell this device apart from an identical one, so it cannot be a target.</p>
          {/if}
          <ul class="parts">
            <li>
              <button
                type="button"
                class="pick"
                disabled={ref === null}
                tabindex={stop(ref, null)}
                aria-current={current(ref, null)}
                data-mirror="target.choose"
                onclick={() => ref !== null && onselect({ device: ref, zone: null })}
                onkeydown={move}
                use:contextMenu={() => menuFor(device, ref, null)}
              >
                <span class="row"><span>Whole device</span><span class="pick-meta">{ledCount(device.descriptor.led_count)}</span></span>
              </button>
            </li>
            {#if device.descriptor.zones.length > 1}
              {#each device.descriptor.zones as zone, index (index)}
                <li>
                  <button
                    type="button"
                    class="pick"
                    disabled={ref === null}
                    tabindex={stop(ref, index)}
                    aria-current={current(ref, index)}
                    data-mirror="target.choose"
                    onclick={() => ref !== null && onselect({ device: ref, zone: index })}
                    onkeydown={move}
                    use:contextMenu={() => menuFor(device, ref, index)}
                  >
                    <span class="row"><span>{names.zone(device, index)}</span><span class="pick-meta">{ledCount(zone.led_count)}</span></span>
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
    padding: var(--space-4) var(--space-3);
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
    padding: 0 var(--space-2) var(--space-3);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .targets > ul {
    display: grid;
    gap: var(--space-4);
  }

  .device {
    display: grid;
    gap: 2px;
  }

  .name {
    font-weight: var(--weight-strong);
    padding: 0 var(--space-2);
  }

  .reason {
    color: var(--text-dim);
    font-size: var(--font-s);
    padding: 0 var(--space-2);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-3);
    padding: 0 var(--space-2);
  }

  .parts {
    display: grid;
    gap: 2px;
    margin-top: var(--space-1);
  }

  .parts .pick {
    min-height: var(--control-h-s);
    padding-block: var(--space-1);
  }

  .row {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
  }
</style>
