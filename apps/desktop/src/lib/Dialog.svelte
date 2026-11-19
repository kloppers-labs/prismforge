<script lang="ts">
  import { onDestroy, onMount, tick, type Snippet } from "svelte";
  import { overlays } from "./overlays.svelte";

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

  onMount(async () => {
    overlays.dialogs += 1;
    await tick();
    focusable()[0]?.focus();
  });
  onDestroy(() => {
    overlays.dialogs -= 1;
    opener?.focus();
  });

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
    padding: var(--space-5);
    background: var(--backdrop);
    z-index: var(--z-dialog);
  }

  .dialog {
    width: min(560px, 100%);
    max-height: 100%;
    overflow-y: auto;
    display: grid;
    gap: var(--space-4);
    padding: var(--space-5);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-m);
    background: var(--surface-1);
    box-shadow: var(--shadow-2);
  }

  .dialog h2 {
    font-size: var(--font-l);
  }

  .body {
    display: grid;
    gap: var(--space-2);
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: flex-end;
  }
</style>
