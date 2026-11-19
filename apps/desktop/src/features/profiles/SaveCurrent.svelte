<script lang="ts">
  import type { ProfilesState } from "./state.svelte";

  let { profiles, field = $bindable() }: { profiles: ProfilesState; field?: HTMLInputElement } = $props();

  let name = $state("");
  let active = $derived(profiles.active?.name ?? null);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (await profiles.saveNew(name.trim())) {
      name = "";
    }
  }
</script>

<section class="save" aria-labelledby="save-title">
  <h3 id="save-title">Save the current lighting</h3>
  <p class="hint">
    {#if active}
      The new profile holds {active} and the changes you made since you applied or saved it. Effects that already ran before that, on devices {active} does not name, are not included.
    {:else}
      The new profile holds the effects, hardware modes, program statements and frame rate in use now.
    {/if}
  </p>
  <form onsubmit={save}>
    <label for="new-profile-name">Profile name</label>
    <input id="new-profile-name" type="text" autocomplete="off" bind:this={field} bind:value={name} />
    <button type="submit" disabled={profiles.working !== null}>{profiles.working === "save" ? "Saving" : "Save as new profile"}</button>
  </form>
  <p class="hint">PrismForge saves profiles on this computer. Nothing is saved to the devices, and effects still stop when PrismForge closes.</p>
</section>

<style>
  .save {
    display: grid;
    gap: 8px;
  }

  form {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  label {
    color: var(--text-dim);
  }

  .hint {
    color: var(--text-faint);
    font-size: var(--font-s);
    max-width: 72ch;
  }
</style>
