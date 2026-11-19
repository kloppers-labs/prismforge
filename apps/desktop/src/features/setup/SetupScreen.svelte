<script lang="ts">
  import { onMount } from "svelte";
  import type { ComponentId } from "../../lib/api/bindings/ComponentId";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { failureText } from "../../lib/errors";
  import Notice from "../../lib/Notice.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import { isTyping } from "../../lib/shell/keyboard";
  import { nativeWindow } from "../../lib/window.svelte";
  import { LiveWatcher } from "../effects/live.svelte";
  import AcrossPanel from "./AcrossPanel.svelte";
  import Canvas from "./Canvas.svelte";
  import { componentColors, liveColors, viewTargets } from "./colors";
  import { AcrossPlayer } from "./player.svelte";
  import StageBar from "./StageBar.svelte";
  import ComponentList from "./ComponentList.svelte";
  import DetectedList from "./DetectedList.svelte";
  import GroupsList from "./GroupsList.svelte";
  import Inspector from "./Inspector.svelte";
  import { componentMenu } from "./menu";
  import LibraryPanel from "./LibraryPanel.svelte";
  import UnwiredList from "./UnwiredList.svelte";
  import { editAction, type EditAction } from "./keys";
  import OutputList from "./OutputList.svelte";
  import { identifyText } from "./present";
  import SetupBar from "./SetupBar.svelte";
  import { outputOf } from "./wiring";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

  let setup = $derived(app.setup);
  let across = $derived(app.across);
  let identified = $state<ComponentId | null>(null);
  let documentShown = $state(true);
  const player = new AcrossPlayer(() => app.effects.catalog?.preview ?? null);
  const watcher = new LiveWatcher();
  let targets = $derived(viewTargets(setup.draft, setup.tab));
  let instance = $derived(across.checked?.instance ?? null);
  let colors = $derived.by(() => {
    if (across.stage === "live") {
      const output = watcher.output;
      return output ? componentColors(setup.draft, (target) => liveColors(output, target)) : null;
    }
    return player.frame ? componentColors(setup.draft, (target) => player.colorsOf(target)) : null;
  });
  let note = $derived.by(() => {
    if (across.stage === "live") {
      if (watcher.refusal) {
        return `The live output cannot be shown. ${watcher.refusal}`;
      }
      return across.running.length === 0 && session.output.scene.assignments.length === 0 ? "No PrismForge effect runs, so nothing is sent and the LEDs show no colour." : null;
    }
    if (instance === null) {
      return "Choose an effect below to see a draft of it across this view.";
    }
    return player.problem;
  });

  $effect(() => {
    setup.follow(session);
    app.groups.follow(session);
  });

  $effect(() => {
    player.show(across.stage === "draft" && instance ? { setup: setup.draft, view: setup.tab, flow: across.flow, instance } : null);
  });

  $effect(() => {
    player.setWindowShown(nativeWindow.visible);
  });

  $effect(() => {
    const visible = across.stage === "live" && nativeWindow.visible && documentShown;
    watcher.watch(visible ? targets : []);
  });

  function onVisibility() {
    documentShown = document.visibilityState !== "hidden";
  }

  onMount(() => {
    void app.effects.load();
    player.open();
    void watcher.open();
    const motion = typeof window.matchMedia === "function" ? window.matchMedia(REDUCED_MOTION) : null;
    watcher.reduced = motion?.matches ?? false;
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      player.close();
      watcher.close();
      document.removeEventListener("visibilitychange", onVisibility);
    };
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
            <StageBar {across} {session} reduced={across.stage === "draft" ? player.reduced : watcher.reduced} slowed={across.stage === "draft" ? player.slowed : watcher.slowed} {note} />
            <Canvas {setup} {colors} live={across.stage === "live"} onaction={act} />
          </div>
          <aside class="side" aria-label="Components and editor">
            <ComponentList {setup} {session} names={app.names} {colors} onaction={act} menu={(id) => componentMenu({ setup, identify }, id)} />
            <Inspector {setup} {session} names={app.names} groups={app.groups} onidentify={identify} />
            {#if identified && setup.selection.includes(identified)}
              <p class="identify" role="status">
                {identifyText(setup.draft, identified, app.names, session)}
                {app.identify?.kind === "running" ? " Identify is running." : app.identify?.kind === "failed" ? ` Identify failed: ${failureText(app.identify.error)}` : ""}
              </p>
            {/if}
          </aside>
        </div>
        <AcrossPanel {across} {setup} {session} names={app.names} catalog={app.effects.catalog} />
        <UnwiredList {setup} {session} names={app.names} />
        <GroupsList {setup} groups={app.groups} />
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

  .drawing {
    display: grid;
    gap: var(--space-2);
    min-width: 0;
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
