<script lang="ts">
  import { flushSync } from "svelte";
  import type { OnConnect } from "../../lib/api/bindings/OnConnect";
  import type { ProfileDetails } from "../../lib/api/bindings/ProfileDetails";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { StoreSummary } from "../../lib/api/bindings/StoreSummary";
  import Dialog from "../../lib/Dialog.svelte";
  import { failureText } from "../../lib/errors";
  import ApplyOnConnect from "./ApplyOnConnect.svelte";
  import ChangeList from "./ChangeList.svelte";
  import { notConnectedText, type Labels } from "./present";
  import SettingsList from "./SettingsList.svelte";
  import type { ProfilesState } from "./state.svelte";

  const EXPORT_PRIVACY =
    "The file holds the profile name, its settings, the model names of its devices and the program names you typed for “Another program”. It leaves out serial numbers, device locations, your names for devices and zones, and the profile's id on this computer.";

  let {
    profiles,
    details,
    summary,
    labels,
    live,
    applying,
    session,
    onConnect,
  }: {
    profiles: ProfilesState;
    details: ProfileDetails;
    summary: StoreSummary;
    labels: Labels;
    live: boolean;
    applying: string | null;
    session: SessionSnapshot | null;
    onConnect: OnConnect;
  } = $props();

  let name = $derived(details.name);
  let active = $derived(summary.active === details.id);
  let marked = $derived(summary.apply_on_connect === details.id);
  let applyReason = $derived(applying ?? (live ? null : notConnectedText(onConnect, session)));
  let unsaved = $derived(summary.unsaved_changes);
  let dialog = $state<"delete" | "export" | null>(null);
  let renaming = $state(false);
  let newName = $state("");
  let renameProblem = $state<string | null>(null);
  let deleteButton: HTMLButtonElement | undefined = $state();
  let exportButton: HTMLButtonElement | undefined = $state();
  let updateButton: HTMLButtonElement | undefined = $state();
  let renameButton: HTMLButtonElement | undefined = $state();
  let renameField: HTMLInputElement | undefined = $state();

  function activeText(): string {
    if (unsaved === null) {
      return "Active in this session. PrismForge could not count the unsaved changes just now.";
    }
    if (unsaved === 0) {
      return "Active in this session. No unsaved changes.";
    }
    return `Active in this session. ${unsaved === 1 ? "1 setting changed" : `${unsaved} settings changed`} since you applied or saved it.`;
  }

  function startRename() {
    newName = name;
    renameProblem = null;
    flushSync(() => (renaming = true));
    renameField?.focus();
  }

  function stopRename() {
    flushSync(() => (renaming = false));
    renameButton?.focus();
  }

  async function rename(event: SubmitEvent) {
    event.preventDefault();
    const failure = await profiles.rename(details.id, newName.trim());
    if (failure === null) {
      stopRename();
    } else {
      renameProblem = failureText(failure);
    }
  }

  function renameKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      stopRename();
    }
  }

  async function confirmDelete() {
    dialog = null;
    await profiles.remove(details.id);
  }

  async function confirmExport() {
    dialog = null;
    await profiles.exportProfile(details.id);
  }
</script>

<article class="profile" aria-labelledby="profile-title">
  <header>
    <h2 id="profile-title">{name}</h2>
    {#if active}
      <p class="sub">{activeText()}</p>
    {/if}
  </header>

  <div class="actions">
    <button
      type="button"
      class="primary"
      disabled={applyReason !== null || profiles.working !== null}
      aria-describedby={applyReason ? "apply-off" : undefined}
      onclick={() => profiles.apply(details.id)}
    >
      {profiles.working === "apply" ? "Applying" : "Apply profile"}
    </button>
    {#if active && live}
      <button
        type="button"
        bind:this={updateButton}
        disabled={applying !== null || profiles.working !== null}
        onclick={() => profiles.previewUpdate(details.id)}
      >
        Save changes to {name}
      </button>
    {/if}
    <button type="button" bind:this={renameButton} onclick={startRename}>Rename</button>
    <button type="button" disabled={profiles.working !== null} onclick={() => profiles.duplicate(details.id)}>Duplicate</button>
    <button type="button" bind:this={exportButton} onclick={() => (dialog = "export")}>Export</button>
    <button type="button" bind:this={deleteButton} disabled={applying !== null} onclick={() => (dialog = "delete")}>Delete</button>
  </div>
  {#if applyReason}
    <p id="apply-off" class="hint">Apply is off. {applyReason}</p>
  {/if}
  {#if live && !active}
    <p class="hint">
      To save changes to this profile, apply it first. PrismForge links your changes to a profile from the moment you apply or save it, until you disconnect, switch between demo mode and OpenRGB, or close PrismForge.
    </p>
  {/if}

  {#if renaming}
    <form class="rename" onsubmit={rename}>
      <label for="profile-name">New name</label>
      <input
        id="profile-name"
        type="text"
        autocomplete="off"
        bind:this={renameField}
        bind:value={newName}
        onkeydown={renameKey}
        aria-invalid={renameProblem ? "true" : undefined}
        aria-describedby={renameProblem ? "rename-problem" : undefined}
      />
      <button type="submit">Save name</button>
      <button type="button" onclick={stopRename}>Cancel</button>
      {#if renameProblem}
        <p id="rename-problem" class="problem" role="alert">{renameProblem}</p>
      {/if}
    </form>
  {/if}

  <ApplyOnConnect on={marked} onchange={(on) => profiles.setApplyOnConnect(on ? details.id : null)} />

  <SettingsList {details} {labels} />
</article>

{#if dialog === "delete"}
  <Dialog title="Delete {name}?" opener={deleteButton} onclose={() => (dialog = null)}>
    <p>The profile file is removed from this computer. This cannot be undone. Exported copies stay as they are.</p>
    {#snippet actions()}
      <button type="button" onclick={() => (dialog = null)}>Keep profile</button>
      <button type="button" class="danger" onclick={confirmDelete}>Delete profile</button>
    {/snippet}
  </Dialog>
{/if}

{#if dialog === "export"}
  <Dialog title="Export {name}" opener={exportButton} onclose={() => (dialog = null)}>
    <p>{EXPORT_PRIVACY}</p>
    <p>Next, choose where to save the file.</p>
    {#snippet actions()}
      <button type="button" onclick={() => (dialog = null)}>Cancel</button>
      <button type="button" class="primary" onclick={confirmExport}>Choose where to save</button>
    {/snippet}
  </Dialog>
{/if}

{#if profiles.pendingUpdate && profiles.pendingUpdate.id === details.id}
  {@const pending = profiles.pendingUpdate}
  <Dialog title="Save changes to {pending.name}?" opener={updateButton} onclose={() => profiles.cancelUpdate()}>
    <p>These changes go into the profile file on this computer. Everything else in the profile stays as it is. Nothing is saved to the devices.</p>
    <ChangeList saved={pending.preview} {labels} />
    {#snippet actions()}
      <button type="button" onclick={() => profiles.cancelUpdate()}>Cancel</button>
      <button type="button" class="primary" onclick={() => profiles.confirmUpdate()}>Save changes</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .profile {
    display: grid;
    gap: 20px;
    align-content: start;
  }

  header {
    display: grid;
    gap: 4px;
  }

  .sub {
    color: var(--text-dim);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .hint {
    color: var(--text-faint);
    font-size: 13px;
    max-width: 72ch;
  }

  .rename {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .rename label {
    color: var(--text-dim);
  }

  .problem {
    flex-basis: 100%;
    color: var(--problem);
  }

  :global(button.danger) {
    border-color: #7a3a33;
    color: var(--problem);
  }
</style>
