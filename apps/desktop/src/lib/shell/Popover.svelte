<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import { overlays } from "../overlays.svelte";
  import Icon from "./Icon.svelte";
  import { contextMenu, type Menu } from "./menu.svelte";
  import { PANEL_MARGIN, panelOffset } from "./placement";

  let {
    id,
    label,
    panelLabel,
    trigger,
    panel,
    focus = "first",
    menu,
  }: {
    id: string;
    label: string;
    panelLabel: string;
    trigger: Snippet;
    panel: Snippet<[() => void]>;
    focus?: "first" | "panel";
    menu?: () => Menu | null;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement | undefined = $state();
  let button: HTMLButtonElement | undefined = $state();
  let body: HTMLDivElement | undefined = $state();
  let offset = $state(0);
  let room = $state<number | null>(null);

  function items(): HTMLElement[] {
    return body ? [...body.querySelectorAll<HTMLElement>("button:not(:disabled), a[href], input:not(:disabled)")] : [];
  }

  async function show(at: "first" | "last" | "panel" = focus) {
    overlays.popover = id;
    open = true;
    offset = 0;
    room = null;
    await tick();
    place();
    const list = items();
    const target = at === "panel" ? body : at === "first" ? list[0] : list[list.length - 1];
    (target ?? body)?.focus();
  }

  function place() {
    if (!root || !body) {
      return;
    }
    const anchor = root.getBoundingClientRect();
    offset = panelOffset({ triggerLeft: anchor.left, panelWidth: body.offsetWidth, viewportWidth: window.innerWidth });
    room = Math.max(window.innerHeight - anchor.bottom - PANEL_MARGIN * 2, 120);
  }

  function hide(returnFocus: boolean) {
    open = false;
    if (overlays.popover === id) {
      overlays.popover = null;
    }
    if (returnFocus) {
      button?.focus();
    }
  }

  function close() {
    hide(true);
  }

  function onTriggerKey(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      void show(focus === "panel" ? "panel" : event.key === "ArrowDown" ? "first" : "last");
    }
  }

  function onPanelKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    const list = items();
    const at = list.indexOf(document.activeElement as HTMLElement);
    const moves: Record<string, number> = { ArrowDown: at + 1, ArrowUp: at === -1 ? list.length - 1 : at - 1, Home: 0, End: list.length - 1 };
    if (event.key in moves && list.length > 0) {
      event.preventDefault();
      const next = (moves[event.key]! + list.length) % list.length;
      list[next]?.focus();
    }
  }

  $effect(() => {
    if (open && overlays.popover !== id) {
      hide(false);
    }
  });

  $effect(() => {
    const panel = body;
    panel?.addEventListener("keydown", onPanelKey);
    return () => panel?.removeEventListener("keydown", onPanelKey);
  });

  function onfocusout(event: FocusEvent) {
    const next = event.relatedTarget;
    if (open && next instanceof Node && !root?.contains(next)) {
      hide(false);
    }
  }

  function onpointerdown(event: PointerEvent) {
    if (open && event.target instanceof Node && !root?.contains(event.target)) {
      hide(false);
    }
  }
</script>

<svelte:window {onpointerdown} onresize={() => open && place()} />

<div class="popover" bind:this={root} {onfocusout}>
  <button
    type="button"
    class="trigger"
    bind:this={button}
    aria-label={label}
    aria-expanded={open}
    aria-controls={id}
    onclick={() => (open ? close() : show())}
    onkeydown={onTriggerKey}
    use:contextMenu={menu}
  >
    {@render trigger()}
    <span class="chevron" class:open><Icon name="chevron" /></span>
  </button>
  {#if open}
    <div
      {id}
      class="panel"
      role="group"
      aria-label={panelLabel}
      tabindex="-1"
      bind:this={body}
      style:left="{offset}px"
      style:max-height={room === null ? null : `${room}px`}
    >
      {@render panel(close)}
    </div>
  {/if}
</div>

<style>
  .popover {
    position: relative;
    min-width: 0;
    max-width: 100%;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    max-width: 100%;
    min-height: var(--control-h);
    padding: var(--space-1) var(--space-2) var(--space-1) var(--space-3);
    background: transparent;
    border-color: transparent;
    font-weight: var(--weight-regular);
    text-align: left;
  }

  .trigger:hover:not(:disabled),
  .trigger[aria-expanded="true"] {
    background: var(--surface-2);
    border-color: var(--line-control);
  }

  .chevron {
    display: inline-flex;
    color: var(--text-dim);
    transition: transform var(--dur-fast) var(--ease-out);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .panel {
    position: absolute;
    top: calc(100% + var(--space-1));
    left: 0;
    z-index: var(--z-popover);
    width: max-content;
    min-width: min(260px, calc(100vw - 2 * var(--space-2)));
    max-width: min(380px, calc(100vw - 2 * var(--space-2)));
    overflow-y: auto;
    overflow-wrap: anywhere;
    display: grid;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
    box-shadow: var(--shadow-2);
  }

  .panel:focus-visible {
    outline-offset: -2px;
  }
</style>
