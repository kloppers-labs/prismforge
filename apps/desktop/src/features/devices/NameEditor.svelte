<script lang="ts">
  import { flushSync } from "svelte";
  import type { Failure } from "../../lib/api/client";
  import { failureText } from "../../lib/errors";

  let {
    kind,
    subject,
    given,
    limit,
    onsave,
  }: {
    kind: "device" | "zone";
    subject: string;
    given: string | null;
    limit: number;
    onsave: (name: string | null) => Promise<Failure | null>;
  } = $props();

  const id = `name-${Math.random().toString(36).slice(2)}`;
  let editing = $state(false);
  let text = $state("");
  let problem = $state<string | null>(null);
  let saving = $state(false);
  let cleared = $state<string | null>(null);
  let renameButton: HTMLButtonElement | undefined = $state();
  let field: HTMLInputElement | undefined = $state();

  let renameLabel = $derived(kind === "device" ? "Rename device" : `Rename zone ${subject}`);
  let renameText = $derived(kind === "device" ? "Rename device" : "Rename");
  let fieldLabel = $derived(kind === "device" ? "Device name" : `Name for zone ${subject}`);
  let clearLabel = $derived(kind === "device" ? "Use the reported name" : `Use the reported name for zone ${subject}`);

  function open() {
    text = given ?? "";
    problem = null;
    flushSync(() => (editing = true));
    field?.focus();
  }

  function close() {
    problem = null;
    flushSync(() => (editing = false));
    renameButton?.focus();
  }

  function check(value: string): string | null {
    if (value.length === 0) {
      return "Enter a name.";
    }
    if ([...value].length > limit) {
      return `Use at most ${limit} characters.`;
    }
    if (/\p{Cc}/u.test(value)) {
      return "Use letters, numbers, spaces and punctuation only.";
    }
    return null;
  }

  async function save() {
    const name = text.trim();
    problem = check(name);
    if (problem !== null || saving) {
      return;
    }
    await send(name);
  }

  async function send(name: string) {
    saving = true;
    const failure = await onsave(name);
    saving = false;
    if (failure === null) {
      close();
    } else {
      problem = failureText(failure);
    }
  }

  async function clear() {
    saving = true;
    const failure = await onsave(null);
    saving = false;
    cleared = failure === null ? null : failureText(failure);
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      save();
    } else if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    save();
  }
</script>

{#if editing}
  <form class="editor" onsubmit={submit}>
    <label for={id}>{fieldLabel}</label>
    <input
      {id}
      type="text"
      autocomplete="off"
      bind:this={field}
      bind:value={text}
      {onkeydown}
      aria-invalid={problem ? "true" : undefined}
      aria-describedby={problem ? `${id}-problem` : `${id}-hint`}
    />
    <button type="submit" disabled={saving}>Save name</button>
    <button type="button" onclick={close}>Cancel</button>
    {#if problem}
      <p id="{id}-problem" class="problem" role="alert">{problem}</p>
    {:else}
      <p id="{id}-hint" class="hint">Up to {limit} characters. The name is kept on this computer, not on the device.</p>
    {/if}
  </form>
{:else}
  <span class="actions">
    <button type="button" class="small" bind:this={renameButton} aria-label={renameLabel} onclick={open}>{renameText}</button>
    {#if given !== null}
      <button type="button" class="small quiet" disabled={saving} aria-label={clearLabel} onclick={clear}>
        {kind === "device" ? clearLabel : "Reported name"}
      </button>
    {/if}
  </span>
  {#if cleared}
    <p class="problem" role="alert">{cleared}</p>
  {/if}
{/if}

<style>
  .editor {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .editor label {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .editor input {
    min-width: 16ch;
  }

  .editor p {
    flex-basis: 100%;
    font-size: var(--font-s);
  }

  .problem {
    color: var(--problem);
  }

  .hint {
    color: var(--text-dim);
  }

  .actions {
    display: inline-flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
