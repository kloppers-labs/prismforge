<script lang="ts">
  import type { Snippet } from "svelte";
  import type { OutputTarget } from "../../lib/api/bindings/OutputTarget";
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { contextMenu, type Menu } from "../../lib/shell/menu.svelte";
  import { deviceKey, hasDirectMode, ledCount, reference } from "../devices/present";
  import { sameTarget, targetMark } from "./present";
  import { choiceBlocked, countText, isMixed, isSelected, options, type TargetOption } from "./selection";

  let {
    session,
    selection,
    shown,
    names,
    onselect,
    ontoggle,
    onrange,
    onall,
    onclear,
    menu,
    groups,
    foot,
  }: {
    session: SessionSnapshot;
    selection: OutputTarget[];
    shown: OutputTarget | null;
    names: Names;
    onselect: (target: OutputTarget) => void;
    ontoggle: (target: OutputTarget) => void;
    onrange: (target: OutputTarget) => void;
    onall: () => void;
    onclear: () => void;
    menu: (device: SessionDevice, target: OutputTarget) => Menu | null;
    groups?: Snippet;
    foot?: Snippet;
  } = $props();

  let list: HTMLElement | undefined = $state();
  let focused = $state<OutputTarget | null>(null);
  let entries = $derived(
    session.devices.map((device, index) => ({ key: deviceKey(device, index), ref: reference(device), device, mark: targetMark(session, device) })),
  );
  let rows = $derived(options(session));
  let count = $derived(selection.length);
  let stop = $derived(
    rows.find((row) => focused !== null && sameTarget(row.target, focused))?.target ??
      rows.find((row) => isSelected(selection, row.target))?.target ??
      rows[0]?.target ??
      null,
  );

  function key(target: OutputTarget): string {
    return `${target.device}|${target.zone ?? "all"}`;
  }

  function rowOf(device: SessionDevice, zone: number | null): TargetOption | undefined {
    const ref = reference(device);
    return rows.find((row) => row.target.device === ref && row.target.zone === zone);
  }

  function focusRow(target: OutputTarget) {
    focused = target;
    list?.querySelector<HTMLElement>(`[data-target="${CSS.escape(key(target))}"]`)?.focus();
  }

  function onkey(event: KeyboardEvent, row: TargetOption) {
    const at = rows.findIndex((each) => sameTarget(each.target, row.target));
    const command = event.ctrlKey || event.metaKey;
    const moves: Record<string, number> = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: rows.length - 1 };
    if (event.key in moves) {
      const next = rows[Math.min(Math.max(moves[event.key] ?? at, 0), rows.length - 1)];
      if (!next) {
        return;
      }
      event.preventDefault();
      if (event.shiftKey && (event.key === "Home" || event.key === "End") && command) {
        onrange(next.target);
      } else if (event.shiftKey && !isSelected(selection, next.target)) {
        ontoggle(next.target);
      }
      focusRow(next.target);
      return;
    }
    if (event.key === " ") {
      event.preventDefault();
      if (event.shiftKey) {
        onrange(row.target);
      } else {
        ontoggle(row.target);
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      onselect(row.target);
    } else if (command && event.key.toLowerCase() === "a") {
      event.preventDefault();
      onall();
    }
  }

  function onclick(event: MouseEvent, row: TargetOption) {
    focused = row.target;
    if (event.target instanceof Element && event.target.closest(".box")) {
      ontoggle(row.target);
    } else if (event.shiftKey) {
      onrange(row.target);
    } else if (event.ctrlKey || event.metaKey) {
      ontoggle(row.target);
    } else {
      onselect(row.target);
    }
  }

</script>

{#snippet option(device: SessionDevice, zone: number | null, label: string, leds: number)}
  {@const row = rowOf(device, zone)}
  {#if row}
    {@const selected = isSelected(selection, row.target)}
    {@const mixed = zone === null && isMixed(selection, row.target.device)}
    {@const current = shown !== null && sameTarget(shown, row.target)}
    <div
      class="pick option"
      role="option"
      tabindex={stop !== null && sameTarget(stop, row.target) ? 0 : -1}
      aria-selected={selected}
      aria-disabled={row.reason !== null ? "true" : undefined}
      aria-current={current ? "true" : undefined}
      aria-describedby={mixed ? `mixed-${key(row.target)}` : row.reason !== null ? `why-${row.target.device}` : undefined}
      aria-keyshortcuts="Shift+F10"
      data-target={key(row.target)}
      data-mirror="target.choose"
      onclick={(event) => onclick(event, row)}
      onkeydown={(event) => onkey(event, row)}
      onfocus={() => (focused = row.target)}
      use:contextMenu={() => menu(device, row.target)}
    >
      <span class="box" class:checked={selected} class:mixed={mixed && !selected} class:off={row.reason !== null} aria-hidden="true">
        {selected ? "✓" : mixed ? "–" : ""}
      </span>
      <span class="row"><span>{label}</span><span class="pick-meta">{ledCount(leds)}</span></span>
      {#if mixed && !selected}<span id="mixed-{key(row.target)}" class="visually-hidden">Some zones selected</span>{/if}
    </div>
  {/if}
{/snippet}

<nav class="targets" aria-labelledby="targets-title">
  {@render groups?.()}
  <div class="head">
    <h2 id="targets-title" class="title">Targets</h2>
    <div class="tools">
      <button type="button" class="small quiet" onclick={onall}>Select all</button>
      <button type="button" class="small quiet" disabled={count === 0} onclick={onclear}>Clear</button>
    </div>
  </div>
  <p id="targets-count" class="count" role="status">{countText(count)}</p>
  <div class="list" role="listbox" aria-multiselectable="true" aria-labelledby="targets-title" aria-describedby="targets-count targets-keys" bind:this={list}>
    {#each entries as { key: entryKey, ref, device, mark } (entryKey)}
      {@const reason = choiceBlocked(session, device, null)}
      <div class="device" role="group" aria-labelledby="target-{entryKey}">
        <p id="target-{entryKey}" class="name">{names.device(device)}</p>
        {#if mark || !device.available || !hasDirectMode(device)}
          <p class="tags">
            {#if mark}<span class="tag {mark.tone}">{mark.text}</span>{/if}
            {#if !device.available}<span class="tag">Unavailable</span>{/if}
            {#if !hasDirectMode(device)}<span class="tag">No Direct Mode</span>{/if}
          </p>
        {/if}
        {#if ref === null}
          <p class="reason">PrismForge cannot tell this device apart from an identical one, so it cannot be a target.</p>
        {:else}
          {#if reason !== null}
            <p id="why-{ref}" class="reason">{reason}</p>
          {/if}
          <div class="parts">
            {@render option(device, null, "Whole device", device.descriptor.led_count)}
            {#if device.descriptor.zones.length > 1}
              {#each device.descriptor.zones as zone, index (index)}
                {@render option(device, index, names.zone(device, index), zone.led_count)}
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
  <p id="targets-keys" class="keys">
    Click a target to light it alone. Space or Ctrl-click adds or removes it; Shift selects a range.
  </p>
  {@render foot?.()}
</nav>

<style>
  .targets {
    border-right: 1px solid var(--line-subtle);
    background: var(--surface-1);
    overflow-y: auto;
    padding: var(--space-4) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: 0 var(--space-2);
  }

  .tools {
    display: flex;
    gap: var(--space-1);
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
  }

  .count,
  .keys {
    padding: 0 var(--space-2);
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .count {
    color: var(--text);
  }

  .list {
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

  .option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: var(--control-h-s);
    padding-block: var(--space-1);
    cursor: pointer;
  }

  .option[aria-disabled="true"] {
    color: var(--text-dim);
  }

  .box {
    flex: none;
    display: inline-grid;
    place-items: center;
    width: var(--space-4);
    height: var(--space-4);
    border: 1px solid var(--line-control);
    border-radius: var(--radius-xs);
    font-size: var(--font-xs);
    line-height: 1;
    color: var(--text);
  }

  .box.checked,
  .box.mixed {
    border-color: var(--accent);
    color: var(--accent);
  }

  .box.off {
    border-style: dashed;
  }

  .row {
    flex: 1;
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
  }

  @media (forced-colors: active) {
    .box.checked,
    .box.mixed {
      border-color: Highlight;
      color: CanvasText;
    }
  }
</style>
