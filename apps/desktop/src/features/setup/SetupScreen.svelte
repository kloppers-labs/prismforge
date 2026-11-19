<script lang="ts">
  import type { ComponentId } from "../../lib/api/bindings/ComponentId";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { failureText } from "../../lib/errors";
  import Notice from "../../lib/Notice.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import { isTyping } from "../../lib/shell/keyboard";
  import Canvas from "./Canvas.svelte";
  import ComponentList from "./ComponentList.svelte";
  import DetectedList from "./DetectedList.svelte";
  import Inspector from "./Inspector.svelte";
  import LibraryPanel from "./LibraryPanel.svelte";
  import UnwiredList from "./UnwiredList.svelte";
  import { editAction, type EditAction } from "./keys";
  import OutputList from "./OutputList.svelte";
  import { identifyText } from "./present";
  import SetupBar from "./SetupBar.svelte";
  import { outputOf } from "./wiring";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let setup = $derived(app.setup);
  let identified = $state<ComponentId | null>(null);

  $effect(() => {
    setup.follow(session);
  });

  function act(action: EditAction) {
    switch (action.kind) {
      case "move":
        setup.moveSelection(action.dx, action.dy);
        break;
      case "turn":
        setup.turnSelection(action.steps);
        break;
      case "flip":
        setup.flipSelection();
        break;
      case "remove":
        setup.takeOut();
        break;
      case "undo":
        setup.undo();
        break;
      case "redo":
        setup.redo();
        break;
      case "save":
        void setup.save();
        break;
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (isTyping(event.target)) {
      return;
    }
    const action = editAction(event);
    if (action && (action.kind === "undo" || action.kind === "redo" || action.kind === "save")) {
      event.preventDefault();
      act(action);
    }
  }

  function identify(id: ComponentId) {
    const output = outputOf(setup.draft, id);
    if (output) {
      identified = id;
      void app.identifyTarget({ device: output.device, zone: output.zone });
    }
  }
</script>

<div class="page" role="presentation" {onkeydown}>
  <SetupBar {setup} {session} names={app.names} />
  <div class="scroll">
    {#if setup.loadProblem}
      <div class="inner">
        <Notice tone="problem" role="alert" title="The setup could not be read.">
          <p>{failureText(setup.loadProblem)}</p>
          {#snippet actions()}
            <button type="button" class="small" onclick={() => setup.load()}>Read the setup again</button>
          {/snippet}
        </Notice>
      </div>
    {:else}
      <div id="setup-panel" class="inner" role="tabpanel" aria-labelledby="tab-{setup.tab}">
        <div class="workspace">
          <div class="drawing">
            <Canvas {setup} colors={null} onaction={act} />
          </div>
          <aside class="side" aria-label="Components and editor">
            <ComponentList {setup} {session} names={app.names} colors={null} onaction={act} menu={() => null} />
            <Inspector {setup} {session} names={app.names} onidentify={identify} />
            {#if identified && setup.selection.includes(identified)}
              <p class="identify" role="status">
                {identifyText(setup.draft, identified, app.names, session)}
                {app.identify?.kind === "running" ? " Identify is running." : app.identify?.kind === "failed" ? ` Identify failed: ${failureText(app.identify.error)}` : ""}
              </p>
            {/if}
          </aside>
        </div>
        <UnwiredList {setup} {session} names={app.names} />
        <OutputList {setup} {session} names={app.names} />
        <DetectedList {setup} {session} names={app.names} />
        <LibraryPanel {setup} />
      </div>
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100%;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
  }

  .scroll {
    overflow: auto;
    container-type: inline-size;
  }

  .inner {
    display: grid;
    gap: var(--space-5);
    padding: var(--space-4) var(--space-6) var(--space-7);
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, 320px);
    gap: var(--space-4);
    align-items: start;
  }

  .side {
    display: grid;
    gap: var(--space-4);
    min-width: 0;
  }

  .identify {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  @container (max-width: 760px) {
    .workspace {
      grid-template-columns: minmax(0, 1fr);
    }

    .inner {
      padding-inline: var(--space-4);
    }
  }
</style>
