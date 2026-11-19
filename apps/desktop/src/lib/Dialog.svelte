<script lang="ts">
  import { onDestroy, onMount, type Snippet } from "svelte";

  let {
    title,
    opener,
    onclose,
    children,
    actions,
  }: {
    title: string;
    opener: HTMLElement | undefined;
    onclose: () => void;
    children: Snippet;
    actions: Snippet;
  } = $props();

  const id = `dialog-${Math.random().toString(36).slice(2)}`;
  let box: HTMLDivElement | undefined = $state();

  function focusable(): HTMLElement[] {
    return box ? [...box.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), [tabindex='0']")] : [];
  }

  onMount(() => focusable()[0]?.focus());
  onDestroy(() => opener?.focus());

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onclose();
      return;
    }
    if (event.key !== "Tab") {
      return;
    }
    const items = focusable();
    const first = items[0];
    const last = items[items.length - 1];
    if (!first || !last) {
      return;
    }
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby={id} tabindex="-1" bind:this={box} {onkeydown}>
    <h2 {id}>{title}</h2>
    <div class="body">{@render children()}</div>
    <div class="actions">{@render actions()}</div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgb(10 11 14 / 72%);
    z-index: 20;
  }

  .dialog {
    width: min(560px, 100%);
    max-height: 100%;
    overflow-y: auto;
    display: grid;
    gap: 16px;
    padding: 24px 26px;
    border: 1px solid var(--line);
    border-radius: var(--radius-m);
    background: var(--ink-1);
  }

  .dialog h2 {
    font-size: 18px;
  }

  .body {
    display: grid;
    gap: 10px;
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
