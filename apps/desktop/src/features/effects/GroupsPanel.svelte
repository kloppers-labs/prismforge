<script lang="ts">
  import type { GroupView } from "../../lib/api/bindings/GroupView";
  import type { MemberStatus } from "../../lib/api/bindings/MemberStatus";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import Details from "../../lib/Details.svelte";
  import Dialog from "../../lib/Dialog.svelte";
  import { failureText } from "../../lib/errors";
  import type { Names } from "../../lib/names";
  import { contextMenu, type Menu } from "../../lib/shell/menu.svelte";
  import { labelOf } from "./batch";
  import { EVERYTHING, groupsNoticeText, stateText, storedLabel, type GroupAsk, type GroupsState } from "./groups.svelte";
  import { selectAll } from "./selection";
  import type { EffectsState } from "./state.svelte";

  let {
    groups,
    effects,
    session,
    names,
    menu,
  }: {
    groups: GroupsState;
    effects: EffectsState;
    session: SessionSnapshot;
    names: Names;
    menu: (group: GroupView | typeof EVERYTHING) => Menu | null;
  } = $props();

  let view = $derived(groups.view);
  let list = $derived(view?.groups ?? []);
  let selected = $derived(effects.targets.length);
  let name = $state("");
  let into = $state<string | null>(null);
  let opener: HTMLElement | undefined = $state();
  let dialog = $state<GroupAsk | null>(null);

  $effect(() => {
    const next = groups.asked;
    if (next) {
      opener = next.from;
      name = next.kind === "rename" ? next.group.name : "";
      into = list[0]?.id ?? null;
      groups.problem = null;
      dialog = next;
      groups.asked = null;
    }
  });

  function memberLabel(member: MemberStatus): string {
    const here = labelOf(session, names, member.target);
    return here !== member.target.device ? here : storedLabel(member);
  }

  function plural(count: number): string {
    return count === 1 ? "1 target" : `${count} targets`;
  }

  function summary(group: GroupView): string {
    const total = group.members.length;
    const here = group.members.filter((member) => member.state === "Resolved").length;
    const count = plural(total);
    return here === total ? count : `${count}, ${total - here} not here today`;
  }

  function open(next: GroupAsk) {
    groups.asked = next;
  }

  async function confirm(event: SubmitEvent) {
    event.preventDefault();
    const current = dialog;
    if (!current) {
      return;
    }
    let done = false;
    if (current.kind === "create") {
      done = await groups.create(name, effects.targets);
    } else if (current.kind === "rename") {
      done = await groups.rename(current.group.id, name);
    } else if (current.kind === "add" && into !== null) {
      done = await groups.add(into, effects.targets);
    } else if (current.kind === "delete") {
      done = await groups.delete(current.group.id);
    }
    if (done) {
      dialog = null;
    }
  }
</script>

<section class="groups" aria-labelledby="groups-title">
  <div class="head">
    <h2 id="groups-title" class="title">Groups</h2>
    <button type="button" class="small quiet" disabled={selected === 0} onclick={(event) => open({ kind: "create", from: event.currentTarget })}>New group…</button>
  </div>
  {#each view?.notices ?? [] as notice (notice.file)}
    <p class="warn">{groupsNoticeText(notice)}</p>
  {/each}
  {#if groups.problem && dialog === null}
    <p class="warn" role="alert">{failureText(groups.problem)}</p>
  {/if}
  <ul>
    <li class="group" use:contextMenu={() => menu(EVERYTHING)} aria-keyshortcuts="Shift+F10">
      <div class="line">
        <span class="name">Everything</span>
        <span class="meta">{plural(selectAll(session).length)}</span>
      </div>
      <button type="button" class="small" data-mirror="group.select" onclick={() => groups.select(EVERYTHING, session, effects)}>Select</button>
    </li>
    {#each list as group (group.id)}
      <li class="group" use:contextMenu={() => menu(group)} aria-keyshortcuts="Shift+F10">
        <div class="line">
          <span class="name">{group.name}</span>
          <span class="meta" class:short={group.members.some((member) => member.state !== "Resolved")}>{summary(group)}</span>
        </div>
        <button type="button" class="small" data-mirror="group.select" onclick={() => groups.select(group, session, effects)}>Select</button>
        <Details summary="Members and actions">
          <ul class="members">
            {#each group.members as member (`${member.target.device}|${member.target.zone}`)}
              <li>
                <span>{memberLabel(member)}: {stateText(member)}</span>
                <button type="button" class="small quiet" disabled={groups.working} onclick={() => groups.remove(group.id, member.target)}>
                  Remove <span class="visually-hidden">{memberLabel(member)}</span>
                </button>
              </li>
            {/each}
          </ul>
          <div class="actions">
            {#if effects.definition}
              <button
                type="button"
                class="small"
                data-mirror="group.apply"
                disabled={!effects.checked?.instance || effects.working !== null || groups.pick(group, session).targets.length === 0}
                onclick={() => (groups.select(group, session, effects), effects.apply())}
              >
                Apply {effects.definition.label} to this group
              </button>
            {/if}
            <button type="button" class="small" disabled={selected === 0 || groups.working} onclick={() => groups.add(group.id, effects.targets)}>
              Add the selection
            </button>
            <button type="button" class="small" data-mirror="group.rename" onclick={(event) => open({ kind: "rename", group, from: event.currentTarget })}>Rename…</button>
            <button type="button" class="small danger" data-mirror="group.delete" onclick={(event) => open({ kind: "delete", group, from: event.currentTarget })}>Delete…</button>
          </div>
        </Details>
      </li>
    {/each}
  </ul>
  <button type="button" class="small quiet" data-mirror="target.add-to-group" disabled={selected === 0 || list.length === 0} onclick={(event) => open({ kind: "add", from: event.currentTarget })}>
    Add selection to a group…
  </button>
  <p class="note" role="status">{groups.note ?? ""}</p>
</section>

{#if dialog?.kind === "create" || dialog?.kind === "rename"}
  <Dialog title={dialog.kind === "rename" ? `Rename ${dialog.group.name}` : "New group"} {opener} onclose={() => (dialog = null)}>
    <form id="group-name-form" class="form" onsubmit={confirm}>
      <label for="group-name">Group name</label>
      <input id="group-name" type="text" autocomplete="off" maxlength={view?.name_limit ?? 60} bind:value={name} />
    </form>
    {#if dialog.kind === "create"}
      <p>The group holds the {selected === 1 ? "target" : `${selected} targets`} selected now. A group only selects targets; it changes no lighting by itself. Groups stay on this computer and are not part of a profile or an export.</p>
    {/if}
    {#if groups.problem}
      <p class="warn" role="alert">{failureText(groups.problem)}</p>
    {/if}
    {#snippet actions()}
      <button type="button" onclick={() => (dialog = null)}>Cancel</button>
      <button type="submit" form="group-name-form" class="primary" disabled={groups.working}>{dialog?.kind === "create" ? "Create group" : "Rename group"}</button>
    {/snippet}
  </Dialog>
{/if}

{#if dialog?.kind === "add"}
  <Dialog title="Add to a group" {opener} onclose={() => (dialog = null)}>
    <form id="group-add-form" class="form" onsubmit={confirm}>
      <fieldset>
        <legend>Add the {selected === 1 ? "selected target" : `${selected} selected targets`} to</legend>
        {#each list as group (group.id)}
          <label class="choice"><input type="radio" name="group-add" value={group.id} bind:group={into} /> {group.name}</label>
        {/each}
      </fieldset>
    </form>
    {#if groups.problem}
      <p class="warn" role="alert">{failureText(groups.problem)}</p>
    {/if}
    {#snippet actions()}
      <button type="button" onclick={() => (dialog = null)}>Cancel</button>
      <button type="submit" form="group-add-form" class="primary" disabled={into === null || groups.working}>Add to group</button>
    {/snippet}
  </Dialog>
{/if}

{#if dialog?.kind === "delete"}
  <Dialog title="Delete {dialog.group.name}?" {opener} onclose={() => (dialog = null)}>
    <p>The group is removed from this computer. Its targets and their lighting stay as they are.</p>
    <form id="group-delete-form" onsubmit={confirm}></form>
    {#snippet actions()}
      <button type="button" onclick={() => (dialog = null)}>Keep group</button>
      <button type="submit" form="group-delete-form" class="danger" disabled={groups.working}>Delete group</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .groups {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--line-subtle);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: 0 var(--space-2);
  }

  .title {
    font-size: var(--font-s);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .group {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-1) var(--space-2);
    padding: 0 var(--space-2);
  }

  .group > :global(.details) {
    grid-column: 1 / -1;
  }

  .line {
    display: grid;
  }

  .name {
    font-weight: var(--weight-strong);
    overflow-wrap: anywhere;
  }

  .meta,
  .note {
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .meta.short,
  .warn {
    color: var(--warning);
  }

  .warn {
    font-size: var(--font-s);
    padding: 0 var(--space-2);
  }

  .note {
    padding: 0 var(--space-2);
  }

  .note:empty {
    display: none;
  }

  .members li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-s);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-top: var(--space-2);
  }

  .form {
    display: grid;
    gap: var(--space-1);
  }

  fieldset {
    display: grid;
    gap: var(--space-1);
    border: 0;
    margin: 0;
    padding: 0;
  }

  .choice {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
</style>
