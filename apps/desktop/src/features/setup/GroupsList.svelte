<script lang="ts">
  import type { GroupView } from "../../lib/api/bindings/GroupView";
  import type { GroupsState } from "../effects/groups.svelte";
  import { storedLabel } from "../effects/groups.svelte";
  import type { SetupState } from "./state.svelte";
  import { outputOf } from "./wiring";

  let { setup, groups }: { setup: SetupState; groups: GroupsState } = $props();

  let list = $derived(groups.view?.groups ?? []);
  let note = $state<string | null>(null);

  function select(group: GroupView) {
    const members = group.members.filter((member) => member.state === "Resolved");
    const picked = setup.placed.filter((id) => {
      const output = outputOf(setup.draft, id);
      return output !== null && members.some((member) => member.target.device === output.device && (member.target.zone === null || member.target.zone === output.zone));
    });
    const left = group.members.filter((member) => !setup.draft.outputs.some((output) => output.device === member.target.device && (member.target.zone === null || member.target.zone === output.zone)));
    setup.select(picked);
    const count = picked.length === 1 ? "1 component" : `${picked.length} components`;
    note = `Selected ${group.name}: ${count}.${left.length > 0 ? ` Not in the setup: ${left.map(storedLabel).join(", ")}.` : ""}`;
  }
</script>

{#if list.length > 0}
  <section class="groups" aria-label="Saved groups">
    <h2>Saved groups</h2>
    <p class="hint">The groups of Effects. Selecting one here selects the components wired to its members in the {setup.view?.name} view.</p>
    <ul>
      {#each list as group (group.id)}
        <li role="group" aria-label={group.name}>
          <span class="name">{group.name}</span>
          <span class="hint">{group.members.length === 1 ? "1 member" : `${group.members.length} members`}</span>
          <button type="button" class="small" onclick={() => select(group)}>Select its components</button>
        </li>
      {/each}
    </ul>
    {#if note}<p class="hint" role="status">{note}</p>{/if}
  </section>
{/if}

<style>
  .groups {
    display: grid;
    gap: var(--space-2);
  }

  h2 {
    font-size: var(--font-m);
  }

  ul {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .name {
    font-weight: var(--weight-strong);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
