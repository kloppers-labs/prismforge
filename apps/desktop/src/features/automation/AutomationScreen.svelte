<script lang="ts">
  import { onMount } from "svelte";
  import type { OpenRgbCheck } from "../../lib/api/bindings/OpenRgbCheck";
  import { failureText, sessionErrorText } from "../../lib/errors";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import RuleItem from "./RuleItem.svelte";
  import { NOTE, UNSUPPORTED } from "./wording";

  let { app }: { app: AppState } = $props();

  let automation = $derived(app.automation);
  let view = $derived(automation.view);
  let supported = $derived(view?.support !== "NotOnThisSystem");

  onMount(() => {
    void app.automation.load();
  });

  function checkText(check: OpenRgbCheck): string | null {
    if (check === "NotChecked") {
      return "Connect to OpenRGB, and PrismForge checks whether OpenRGB's own profile settings act on the same events.";
    }
    if (check === "CannotTell") {
      return "PrismForge cannot check OpenRGB's own profile settings here, so it cannot tell whether OpenRGB also acts on these events.";
    }
    if ("Failed" in check) {
      return `PrismForge could not read OpenRGB's own profile settings. ${sessionErrorText(check.Failed.error)}`;
    }
    return null;
  }
</script>

<div class="automation">
  <header class="intro">
    <h1>Automation</h1>
    <p><strong>{NOTE}</strong> Each rule changes your lighting only after you switch it on, and only for its own event.</p>
    {#if view?.store === "Demo"}
      <p>These are the rules of demo mode. The rules for your OpenRGB devices are kept apart.</p>
    {/if}
  </header>

  {#if automation.loadFailure}
    <div class="notice problem" role="alert">
      <ToneIcon tone="problem" />
      <p>{failureText(automation.loadFailure)}</p>
    </div>
  {/if}

  {#if view}
    {#if !supported}
      <div class="notice warning" role="status">
        <ToneIcon tone="warning" />
        <p>{UNSUPPORTED}</p>
      </div>
    {/if}
    {#if checkText(view.openrgb)}
      <p class="check">{checkText(view.openrgb)}</p>
    {/if}
    <ol class="rules" aria-label="Rules">
      {#each view.rules as rule (rule.trigger)}
        <RuleItem {rule} profiles={view.profiles} {automation} {supported} />
      {/each}
    </ol>
  {/if}
</div>

<style>
  .automation {
    max-width: 820px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-4);
  }

  .intro {
    display: grid;
    gap: var(--space-1);
  }

  .intro p,
  .check {
    color: var(--text-dim);
    max-width: 72ch;
  }

  .intro strong {
    color: var(--text);
  }

  .check {
    font-size: var(--font-s);
  }

  .notice {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    max-width: 72ch;
  }

  .rules {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
  }

  @media (max-width: 700px) {
    .automation {
      padding: var(--space-4);
    }
  }
</style>
