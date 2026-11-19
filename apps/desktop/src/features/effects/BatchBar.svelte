<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { namesLine, replaced } from "./batch";
  import { canStop, type Mode } from "./present";
  import AcrossMode from "../setup/AcrossMode.svelte";
  import type { EffectsState } from "./state.svelte";

  let { effects, session, names }: { effects: EffectsState; session: SessionSnapshot; names: Names } = $props();

  let targets = $derived(effects.targets);
  let count = $derived(targets.length);
  let label = (id: string) => effects.catalog?.effects.find((effect) => effect.id === id)?.label ?? id;
  let stoppable = $derived(targets.some((target) => canStop(session, target)));
  let mode: Mode = $derived.by(() => {
    if (effects.blocked !== null) {
      return { name: "Off", text: `Effects are off. ${effects.blocked}` };
    }
    if (effects.canAutoApply) {
      return { name: "Live", text: `Changes go to all ${count} targets as you make them.` };
    }
    if (effects.effect === null) {
      return { name: "Draft", text: "Choose an effect. Your devices stay as they are until you apply it." };
    }
    if (effects.differs.length > 0) {
      return { name: "Draft", text: `Some settings differ between the targets, marked Mixed. Apply sends the values shown to all ${count} targets.` };
    }
    if (effects.allRun) {
      return { name: "Draft", text: `${label(effects.effect)} runs on every target. Press Apply to send your changes.` };
    }
    const others = replaced(session, targets, effects.effect);
    if (others > 0) {
      return { name: "Draft", text: `Apply replaces the effects that run on ${others} of ${count} targets. Your devices stay as they are until then.` };
    }
    return { name: "Draft", text: "Your devices stay as they are until you press Apply." };
  });
</script>

<section class="bar" class:live={mode.name === "Live"} aria-label="Effect actions">
  <div class="title">
    <h2 id="target-title">{count} targets</h2>
    <p class="sub">{namesLine(session, names, targets)}</p>
  </div>
  <div class="actions">
    <button
      type="button"
      class="primary"
      data-mirror="effect.apply"
      disabled={effects.blocked !== null || !effects.checked?.instance || effects.working !== null}
      aria-describedby="edit-mode"
      onclick={() => effects.apply()}
    >
      {effects.working === "apply" ? "Applying…" : `Apply to ${count} targets`}
    </button>
    <button
      type="button"
      data-mirror="target.stop"
      disabled={!stoppable || effects.working !== null || effects.blocked !== null}
      aria-describedby={stoppable ? undefined : "stop-none"}
      onclick={() => effects.stop()}
    >
      {effects.working === "stop" ? "Stopping…" : `Stop on ${count} targets`}
    </button>
    <p id="edit-mode" class="mode">
      <span class="badge {mode.name.toLowerCase()}">{mode.name}</span>
      <span>{mode.text}</span>
    </p>
  </div>
  {#if typeof effects.mode === "object"}
    <AcrossMode view={effects.mode.AcrossSetup.view} flow={effects.mode.AcrossSetup.flow} />
  {/if}
  {#if !stoppable}
    <p id="stop-none" class="note">Stop is off. No PrismForge effect runs on these targets.</p>
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

  .bar.live {
    border-bottom: 3px solid var(--ok);
  }

  .title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 var(--space-3);
  }

  .sub,
  .note {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .mode {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin-left: var(--space-2);
    flex: 1 1 18rem;
    color: var(--text-dim);
  }

  .badge {
    flex: none;
    padding: 0 var(--space-2);
    border-radius: var(--radius-xs);
    border: 1px solid var(--line-control);
    font-size: var(--font-xs);
    font-weight: var(--weight-heading);
    color: var(--text);
  }

  .badge.live {
    border-color: var(--ok);
    color: var(--ok);
  }

  .badge.off {
    border-style: dashed;
  }
</style>
