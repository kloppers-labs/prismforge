<script lang="ts">
  import { untrack } from "svelte";
  import { overlays } from "../overlays.svelte";
  import { boxOf, groups, inView, isMenuKey, MENU_OVERLAY, menus, type MenuItem } from "./menu.svelte";
  import { menuPlacement } from "./placement";

  const id = `menu-${Math.random().toString(36).slice(2)}`;
  let box: HTMLDivElement | undefined = $state();
  let opened = $derived(menus.current);
  let menu = $derived(opened ? opened.build() : null);
  let sections = $derived(menu ? groups(menu.items) : []);
  let checkable = $derived(menu?.items.some((item) => item.checked !== undefined) ?? false);
  let focusedId: string | null = null;
  let shown: object | null = null;

  function place() {
    const current = opened;
    if (!current || !box) {
      return;
    }
    const now = current.origin.isConnected ? boxOf(current.origin) : current.originBox;
    const dx = now.left - current.originBox.left;
    const dy = now.top - current.originBox.top;
    const anchor = { left: current.anchor.left + dx, right: current.anchor.right + dx, top: current.anchor.top + dy, bottom: current.anchor.bottom + dy };
    const spot = menuPlacement(anchor, { width: box.offsetWidth, height: box.offsetHeight }, { width: window.innerWidth, height: window.innerHeight });
    box.style.left = `${spot.left}px`;
    box.style.top = `${spot.top}px`;
  }

  function firstEnabled(): HTMLElement | undefined {
    const list = entries();
    return list.find((entry) => entry.getAttribute("aria-disabled") !== "true") ?? list[0];
  }

  $effect(() => {
    const current = opened;
    if (!current || !box) {
      return;
    }
    untrack(() => {
      shown = current;
      focusedId = null;
      place();
      (menu?.focus === "menu" ? box : (firstEnabled() ?? box))?.focus();
    });
  });

  $effect(() => {
    const current = opened;
    const now = menu;
    if (!current) {
      return;
    }
    if (!now || now.items.length === 0 || !current.origin.isConnected) {
      menus.close(false);
      return;
    }
    if (!box || shown !== current) {
      return;
    }
    place();
    if (!box.contains(document.activeElement)) {
      const same = entries().find((entry) => entry.dataset.item === focusedId);
      (same ?? firstEnabled() ?? box).focus();
    }
  });

  $effect(() => {
    if (opened && overlays.popover !== MENU_OVERLAY) {
      menus.close(false);
    }
  });

  function entries(): HTMLElement[] {
    return box ? [...box.querySelectorAll<HTMLElement>("[data-item]")] : [];
  }

  function activate(item: MenuItem) {
    const current = opened;
    if (!current || item.reason) {
      return;
    }
    menus.close(true);
    void item.run(current.origin);
  }

  function step(list: HTMLElement[], at: number, key: string): number | null {
    const last = list.length - 1;
    if (key === "ArrowDown") {
      return at === -1 || at === last ? 0 : at + 1;
    }
    if (key === "ArrowUp") {
      return at <= 0 ? last : at - 1;
    }
    return key === "Home" ? 0 : key === "End" ? last : null;
  }

  function typeAhead(list: HTMLElement[], at: number, key: string): number | null {
    const letter = key.toLocaleLowerCase();
    for (let offset = 1; offset <= list.length; offset += 1) {
      const index = (at + offset + list.length) % list.length;
      if (list[index]?.dataset.label?.toLocaleLowerCase().startsWith(letter)) {
        return index;
      }
    }
    return null;
  }

  function onkeydown(event: KeyboardEvent) {
    const current = opened;
    if (!current) {
      return;
    }
    const list = entries();
    const at = list.indexOf(document.activeElement as HTMLElement);
    if (event.key === "Escape" || event.key === "Tab") {
      event.preventDefault();
      event.stopPropagation();
      menus.close(true);
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      const item = at === -1 ? null : itemAt(at);
      if (item) {
        activate(item);
      }
      return;
    }
    if (isMenuKey(event)) {
      event.preventDefault();
      return;
    }
    const printable = event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey;
    const next = printable ? typeAhead(list, at, event.key) : step(list, at, event.key);
    if (printable || next !== null) {
      event.preventDefault();
    }
    if (next !== null) {
      list[next]?.focus();
    }
  }

  function itemAt(index: number): MenuItem | null {
    return sections.flat()[index] ?? null;
  }

  function onfocusin(event: FocusEvent) {
    if (event.target instanceof HTMLElement && event.target.dataset.item) {
      focusedId = event.target.dataset.item;
    }
  }

  function onfocusout(event: FocusEvent) {
    const next = event.relatedTarget;
    if (next instanceof Node) {
      if (!box?.contains(next)) {
        menus.close(false);
      }
      return;
    }
    queueMicrotask(() => box?.contains(document.activeElement) || menus.close(false));
  }

  function onscroll(event: Event) {
    const current = opened;
    const scroller = event.target;
    if (!current || (scroller instanceof Node && box?.contains(scroller))) {
      return;
    }
    if (scroller instanceof Node && !(scroller instanceof Document) && !scroller.contains(current.origin)) {
      return;
    }
    if (inView(current.origin)) {
      place();
    } else {
      menus.close(false);
    }
  }

  function onpointerdown(event: PointerEvent) {
    if (opened && !(event.target instanceof Node && box?.contains(event.target))) {
      menus.close(false);
    }
  }

  function reasonId(index: number, offset: number): string {
    return `${id}-${index}-${offset}-reason`;
  }
</script>

<svelte:window onpointerdowncapture={onpointerdown} onresize={() => requestAnimationFrame(place)} onscrollcapture={onscroll} />

{#if opened}
  <div
    {id}
    class="menu"
    class:checkable
    role="menu"
    aria-label={menu?.label}
    tabindex="-1"
    bind:this={box}
    {onkeydown}
    {onfocusin}
    {onfocusout}
    oncontextmenu={(event) => event.preventDefault()}
  >
    {#each sections as group, index (index)}
      {#if index > 0}<div class="line" role="separator"></div>{/if}
      {#each group as item, offset (item.id)}
        <div
          class="item"
          class:danger={item.danger}
          role={item.checked === undefined ? "menuitem" : "menuitemcheckbox"}
          tabindex="-1"
          data-item={item.id}
          data-label={item.label}
          aria-checked={item.checked === undefined ? undefined : item.checked}
          aria-disabled={item.reason ? "true" : undefined}
          aria-labelledby="{id}-{index}-{offset}-label"
          aria-describedby={item.reason ? reasonId(index, offset) : undefined}
          onclick={() => activate(item)}
          onpointermove={(event) => event.currentTarget !== document.activeElement && event.currentTarget.focus()}
        >
          {#if checkable}<span class="check" class:box={item.checked !== undefined} aria-hidden="true"></span>{/if}
          <span class="label" id="{id}-{index}-{offset}-label">{item.label}</span>
          {#if item.reason}<span class="reason" id={reasonId(index, offset)}>{item.reason}</span>{/if}
        </div>
      {/each}
    {/each}
  </div>
{/if}

<style>
  .menu {
    position: fixed;
    left: 0;
    top: 0;
    z-index: var(--z-popover);
    display: grid;
    min-width: min(var(--menu-w-min), calc(100vw - 2 * var(--space-2)));
    max-width: min(var(--menu-w-max), calc(100vw - 2 * var(--space-2)));
    max-height: calc(100vh - 2 * var(--space-2));
    overflow-y: auto;
    padding: var(--space-1);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
    box-shadow: var(--shadow-2);
    font-size: var(--font-m);
  }

  .menu:focus-visible {
    outline-offset: -2px;
  }

  .item {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    align-content: center;
    gap: 0 var(--space-2);
    min-height: var(--control-h);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-s);
    color: var(--text);
    cursor: pointer;
    overflow-wrap: anywhere;
  }

  .checkable .item {
    grid-template-columns: var(--space-4) minmax(0, 1fr);
  }

  .item:focus {
    background: var(--surface-3);
  }

  .item:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }

  .item.danger {
    color: var(--problem);
  }

  .item[aria-disabled="true"] {
    color: var(--text-faint);
    cursor: not-allowed;
  }

  .label {
    grid-column: -2;
  }

  .reason {
    grid-column: -2;
    color: var(--text-dim);
    font-size: var(--font-xs);
  }

  .check {
    grid-row: 1;
    width: var(--space-3);
    height: var(--space-3);
    align-self: center;
  }

  .check.box {
    border: 1px solid var(--line-control);
    border-radius: var(--radius-xs);
  }

  [aria-checked="true"] .check {
    border-color: var(--accent);
    background: var(--accent);
    box-shadow: inset 0 0 0 2px var(--surface-1);
  }

  .line {
    height: 1px;
    margin: var(--space-1) var(--space-2);
    background: var(--line-subtle);
  }

  @media (forced-colors: active) {
    .menu {
      border-color: CanvasText;
    }

    .line {
      background: CanvasText;
    }

    .item[aria-disabled="true"] {
      color: GrayText;
    }

    .item:focus {
      outline: 2px solid Highlight;
      outline-offset: -2px;
    }

    .check.box {
      border-color: CanvasText;
    }

    [aria-checked="true"] .check {
      forced-color-adjust: none;
      background: Highlight;
      box-shadow: inset 0 0 0 2px Canvas;
    }
  }
</style>
