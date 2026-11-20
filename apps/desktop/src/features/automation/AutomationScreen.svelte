<script lang="ts">
  import { onMount } from "svelte";
  import type { OpenRgbCheck } from "../../lib/api/bindings/OpenRgbCheck";
  import { failureText, sessionErrorText } from "../../lib/errors";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import type { AppState } from "../../lib/state/app.svelte";
  import RuleItem from "./RuleItem.svelte";
  import { NOTE, isSystemEvent, runNoticeText, supportText, systemEventsBlocked } from "./wording";

  let { app }: { app: AppState } = $props();

  let automation = $derived(app.automation);
  let view = $derived(automation.view);
  let blocked = $derived(view ? systemEventsBlocked(view.support) : false);
  let support = $derived(view ? supportText(view.support) : null);
  let prismforgeRules = $derived(view ? view.rules.filter((rule) => !isSystemEvent(rule.trigger)) : []);
  let systemRules = $derived(view ? view.rules.filter((rule) => isSystemEvent(rule.trigger)) : []);

  onMount(() => {
    void opened();
  });

  async function opened() {
    await app.automation.load();
    await app.automation.markSeen(app.automation.unseenFailures.map((run) => run.trigger));
  }

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
    <p>{NOTE}</p>
  </header>

  {#if automation.loadFailure}
    <div class="notice problem" role="alert">
      <ToneIcon tone="problem" />
      <p>{failureText(automation.loadFailure)}</p>
    </div>
  {/if}

  {#if view}
    {#each view.notices as notice, index (index)}
      <div class="notice warning">
        <ToneIcon tone="warning" />
        <p>{runNoticeText(notice)}</p>
      </div>
    {/each}
    <section class="group" aria-labelledby="automation-prismforge">
      <h2 id="automation-prismforge">PrismForge</h2>
      <ul class="rules">
        {#each prismforgeRules as rule (rule.trigger)}
          <RuleItem {rule} profiles={view.profiles} {automation} available />
        {/each}
      </ul>
    </section>
    <section class="group" aria-labelledby="automation-system">
      <h2 id="automation-system">System</h2>
      {#if support}
        <p class="note">
          {#if view.support !== "NotOnThisSystem"}<ToneIcon tone="warning" />{/if}
          <span>{support}</span>
        </p>
      {/if}
      <ul class="rules">
        {#each systemRules as rule (rule.trigger)}
          <RuleItem {rule} profiles={view.profiles} {automation} available={!blocked} />
        {/each}
      </ul>
    </section>
    {#if checkText(view.openrgb)}
      <p class="check">{checkText(view.openrgb)}</p>
    {/if}
  {/if}
</div>

<style>
  .automation {
    max-width: 820px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-6) var(--space-7);
    display: grid;
    gap: var(--space-5);
  }

  .intro {
    display: grid;
    gap: var(--space-1);
  }

  .intro p,
  .check,
  .note {
    color: var(--text-dim);
    max-width: 72ch;
  }

  .check,
  .note {
    font-size: var(--font-s);
  }

  .notice,
  .note {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    max-width: 72ch;
  }

  .group {
    display: grid;
    gap: var(--space-2);
  }

  .group h2 {
    font-size: var(--font-m);
    font-weight: var(--weight-strong);
    color: var(--text-dim);
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
