<script lang="ts">
  import type { ProfileChoice } from "../../lib/api/bindings/ProfileChoice";
  import type { RuleView } from "../../lib/api/bindings/RuleView";
  import Details from "../../lib/Details.svelte";
  import { failureText } from "../../lib/errors";
  import Switch from "../../lib/Switch.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import type { AutomationState } from "./state.svelte";
  import {
    LIGHTS_OFF,
    actionKind,
    endingProfileText,
    isSystemEvent,
    openRgbAlsoText,
    profileOf,
    profileProblemText,
    promiseText,
    runLines,
    runTime,
    sentence,
    type ActionKind,
  } from "./wording";

  let {
    rule,
    profiles,
    automation,
    blocked,
  }: { rule: RuleView; profiles: ProfileChoice[]; automation: AutomationState; blocked: string | null } = $props();

  let id = $derived(`rule-${rule.trigger.toLowerCase()}`);
  let chosen = $state<ActionKind | null>(null);
  let stored = $derived(actionKind(rule.action));
  let kind = $derived(chosen ?? stored);
  let profile = $derived(profileOf(rule.action));
  let known = $derived(profile !== null && profiles.some((choice) => choice.id === profile));
  let available = $derived(blocked === null || !isSystemEvent(rule.trigger));
  let busy = $derived(automation.pending === rule.trigger);
  let locked = $derived(!available || busy || stored === "Newer");
  let failure = $derived(automation.failure?.trigger === rule.trigger ? failureText(automation.failure.error) : null);
  let problem = $derived(profileProblemText(rule));
  let ending = $derived(endingProfileText(rule.trigger, rule.action));

  $effect(() => {
    void rule.action;
    chosen = null;
  });

  function chooseAction(event: Event) {
    const next = (event.currentTarget as HTMLSelectElement).value as ActionKind;
    if (next === "ApplyProfile") {
      chosen = stored === "ApplyProfile" ? null : "ApplyProfile";
      return;
    }
    chosen = null;
    void automation.setRule(rule.trigger, rule.enabled, next === "LightsOff" ? "LightsOff" : "Nothing");
  }

  function chooseProfile(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (value !== "") {
      void automation.setRule(rule.trigger, rule.enabled, { ApplyProfile: { profile: value } });
    }
  }
</script>

<li class="rule" class:on={rule.enabled}>
  <h2 id="{id}-title">{sentence(rule)}</h2>
  <div class="controls">
    <Switch
      label="Use this rule"
      labelledby="{id}-title"
      on={rule.enabled}
      disabled={locked}
      describedby="{id}-promise"
      onchange={(on) => automation.setRule(rule.trigger, on, rule.action)}
    />
    <label class="field">
      <span id="{id}-action">Action</span>
      <select value={kind} disabled={locked} aria-labelledby="{id}-title {id}-action" onchange={chooseAction}>
        <option value="Nothing">Do nothing</option>
        <option value="LightsOff">Switch the lights off</option>
        <option value="ApplyProfile">Apply a profile</option>
        {#if stored === "Newer"}
          <option value="Newer" disabled>Set by a newer PrismForge</option>
        {/if}
      </select>
    </label>
    {#if kind === "ApplyProfile"}
      <label class="field">
        <span id="{id}-profile">Profile</span>
        <select
          value={known && chosen === null ? profile : ""}
          disabled={locked || profiles.length === 0}
          aria-labelledby="{id}-title {id}-profile"
          onchange={chooseProfile}
        >
          <option value="" disabled>{profiles.length === 0 ? "No profiles saved yet" : "Choose a profile"}</option>
          {#each profiles as choice (choice.id)}
            <option value={choice.id}>{choice.name}</option>
          {/each}
        </select>
      </label>
    {/if}
  </div>

  <p id="{id}-promise" class="promise">{promiseText(rule.trigger, rule.action)}</p>
  {#if ending}
    <p class="hint">{ending}</p>
  {/if}
  {#if !available}
    <p class="hint unavailable">{blocked}</p>
  {/if}
  {#if kind === "LightsOff"}
    <Details summary="What switching the lights off does">
      <p>{LIGHTS_OFF}</p>
    </Details>
  {/if}
  {#if problem}
    <div class="notice problem"><ToneIcon tone="problem" /><p>{problem}</p></div>
  {/if}
  {#each rule.openrgb_also as act (act.setting)}
    <div class="notice warning"><ToneIcon tone="warning" /><p>{openRgbAlsoText(act)}</p></div>
  {/each}
  {#if failure}
    <p class="failure" role="alert">{failure}</p>
  {/if}
  {#if rule.last_run}
    <section class="last" aria-label="Last run">
      <p class="when">Last run: {runTime(rule.last_run)}</p>
      {#each runLines(rule.last_run) as line, index (index)}
        <p>{line}</p>
      {/each}
    </section>
  {/if}
</li>

<style>
  .rule {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-4) 0;
    border-top: 1px solid var(--line-subtle);
  }

  h2 {
    font-size: var(--font-l);
    color: var(--text-dim);
  }

  .on h2 {
    color: var(--text);
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: var(--space-2) var(--space-4);
  }

  .field {
    display: grid;
    gap: var(--space-1);
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .field select {
    min-width: 14rem;
    max-width: 100%;
  }

  .promise,
  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
    max-width: 72ch;
  }

  .unavailable {
    color: var(--text);
  }

  .notice {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    max-width: 72ch;
    font-size: var(--font-s);
  }

  .failure {
    color: var(--problem);
    font-size: var(--font-s);
  }

  .last {
    display: grid;
    gap: var(--space-1);
    max-width: 72ch;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--line-control);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    font-size: var(--font-s);
  }

  .when {
    font-weight: var(--weight-strong);
  }
</style>
