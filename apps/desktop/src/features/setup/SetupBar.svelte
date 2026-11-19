<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { failureText } from "../../lib/errors";
  import type { Names } from "../../lib/names";
  import { isMac } from "../../lib/platform";
  import { VIEWS } from "./edits";
  import { setupProblemText } from "./present";
  import type { SetupState } from "./state.svelte";

  let { setup, session, names }: { setup: SetupState; session: SessionSnapshot; names: Names } = $props();

  const TABS = [
    { id: VIEWS.pc, label: "PC" },
    { id: VIEWS.desk, label: "Desk" },
  ];

  let problem = $derived.by(() => {
    const failure = setup.saveProblem;
    if (failure === null) {
      return null;
    }
    if (typeof failure === "object" && "Store" in failure && "InvalidSetup" in failure.Store) {
      return setupProblemText(failure.Store.InvalidSetup.problem, setup.draft, names, session);
    }
    return failureText(failure);
  });
  let command = isMac() ? "Command" : "Ctrl";

  function onkey(event: KeyboardEvent, index: number) {
    const moves: Record<string, number> = { ArrowRight: index + 1, ArrowLeft: index - 1, Home: 0, End: TABS.length - 1 };
    if (!(event.key in moves)) {
      return;
    }
    event.preventDefault();
    const next = TABS[((moves[event.key] ?? index) + TABS.length) % TABS.length]!;
    setup.show(next.id);
    document.getElementById(`tab-${next.id}`)?.focus();
  }
</script>

<section class="bar" aria-label="Setup actions">
  <div class="top">
    <h1>Setup</h1>
    <div class="tabs" role="tablist" aria-label="Views">
      {#each TABS as tab, index (tab.id)}
        <button
          type="button"
          role="tab"
          id="tab-{tab.id}"
          aria-selected={setup.tab === tab.id}
          aria-controls="setup-panel"
          tabindex={setup.tab === tab.id ? 0 : -1}
          onclick={() => setup.show(tab.id)}
          onkeydown={(event) => onkey(event, index)}
        >
          {tab.label}
        </button>
      {/each}
    </div>
    <p class="state" class:unsaved={setup.dirty} role="status">
      <span class="mark" aria-hidden="true">{setup.dirty ? "◆" : "✓"}</span>
      {setup.dirty ? "Unsaved changes" : setup.justSaved ? "Saved just now" : "Saved"}
    </p>
    <div class="actions">
      <button type="button" class="small" disabled={setup.undoLabel === null} aria-describedby="undo-what" aria-keyshortcuts="{command}+Z" onclick={() => setup.undo()}>Undo</button>
      <span id="undo-what" class="visually-hidden">{setup.undoLabel ? `Undoes: ${setup.undoLabel}` : "Nothing to undo"}</span>
      <button type="button" class="small" disabled={setup.redoLabel === null} aria-describedby="redo-what" aria-keyshortcuts="{command}+Shift+Z" onclick={() => setup.redo()}>Redo</button>
      <span id="redo-what" class="visually-hidden">{setup.redoLabel ? `Redoes: ${setup.redoLabel}` : "Nothing to redo"}</span>
      <button type="button" class="small" disabled={!setup.dirty || setup.saving} onclick={() => setup.discard()}>Discard changes</button>
      <button type="button" class="small primary" disabled={!setup.dirty || setup.saving} aria-keyshortcuts="{command}+S" onclick={() => setup.save()}>
        {setup.saving ? "Saving…" : "Save setup"}
      </button>
    </div>
  </div>
  <p class="note">Undo and redo change this drawing only. Your lights stay as they are. Save setup keeps the drawing on this computer.</p>
  {#if problem}
    <div class="problem" role="alert">
      <strong>The setup was not saved.</strong>
      {problem}
    </div>
  {/if}
</section>

<style>
  .bar {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-6);
    border-bottom: 1px solid var(--line-subtle);
    background: var(--surface-0);
  }

  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-4);
  }

  h1 {
    font-size: var(--font-xl);
  }

  .tabs {
    display: flex;
    gap: var(--space-1);
  }

  [role="tab"] {
    min-width: 72px;
  }

  [role="tab"][aria-selected="true"] {
    background: var(--surface-3);
    border-color: var(--accent);
    font-weight: var(--weight-strong);
    box-shadow: inset 0 -3px 0 var(--accent);
  }

  .state {
    display: inline-flex;
    gap: var(--space-1);
    align-items: center;
    color: var(--ok);
    font-size: var(--font-s);
  }

  .state.unsaved {
    color: var(--warning);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-left: auto;
  }

  .note {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .problem {
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--problem);
    background: var(--problem-surface);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  @media (forced-colors: active) {
    [role="tab"][aria-selected="true"] {
      border-color: Highlight;
      box-shadow: inset 0 -3px 0 Highlight;
    }
  }
</style>
