<script lang="ts">
  import Dialog from "../../lib/Dialog.svelte";
  import type { ProfilesState } from "./state.svelte";

  let { profiles, opener, onclose }: { profiles: ProfilesState; opener: HTMLElement | undefined; onclose: () => void } = $props();

  let name = $state("");
  let active = $derived(profiles.active?.name ?? null);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (await profiles.saveNew(name.trim())) {
      name = "";
      onclose();
    }
  }
</script>

<Dialog title="Save the current lighting" {opener} {onclose}>
  <form id="save-current" class="save" onsubmit={save}>
    <label for="new-profile-name">Profile name</label>
    <input id="new-profile-name" type="text" autocomplete="off" bind:value={name} />
  </form>
  <p>
    {#if active}
      The new profile holds {active} and the changes you made since you applied or saved it. Effects that already ran before that, on devices {active} does not name, are not included.
    {:else}
      The new profile holds the effects, hardware modes, program statements and frame rate in use now.
    {/if}
  </p>
  <p>PrismForge saves profiles on this computer. Nothing is saved to the devices, and effects still stop when PrismForge closes.</p>
  {#snippet actions()}
    <button type="button" onclick={onclose}>Cancel</button>
    <button type="submit" form="save-current" class="primary" disabled={profiles.working !== null}>
      {profiles.working === "save" ? "Saving…" : "Save as new profile"}
    </button>
  {/snippet}
</Dialog>

<style>
  .save {
    display: grid;
    gap: var(--space-1);
  }

  label {
    color: var(--text);
  }
</style>
