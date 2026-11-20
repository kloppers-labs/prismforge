<script lang="ts">
  import type { ProfileChoice } from "../../lib/api/bindings/ProfileChoice";
  import type { RuleView } from "../../lib/api/bindings/RuleView";
  import { failureText } from "../../lib/errors";
  import Switch from "../../lib/Switch.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import type { AutomationState } from "./state.svelte";
  import {
    LIGHTS_OFF,
    actionKind,
    endingProfileText,
    eventLabel,
    openRgbAlsoShortText,
    openRgbAlsoText,
    profileOf,
    profileProblemText,
    promiseText,
    runFailed,
    runLines,
    runTime,
    sentence,
    shortPromiseText,
    type ActionKind,
  } from "./wording";

  let {
    rule,
    profiles,
    automation,
    available,
  }: { rule: RuleView; profiles: ProfileChoice[]; automation: AutomationState; available: boolean } = $props();

  let id = $derived(`rule-${rule.trigger.toLowerCase()}`);
  let label = $derived(eventLabel(rule.trigger));
  let event = $derived(label.charAt(0).toLowerCase() + label.slice(1));
  let chosen = $state<ActionKind | null>(null);
  let stored = $derived(actionKind(rule.action));
  let kind = $derived(chosen ?? stored);
  let profile = $derived(profileOf(rule.action));
  let known = $derived(profile !== null && profiles.some((choice) => choice.id === profile));
  let busy = $derived(automation.pending === rule.trigger);
  let locked = $derived(!available || busy || stored === "Newer");
  let failure = $derived(automation.failure?.trigger === rule.trigger ? failureText(automation.failure.error) : null);
  let problem = $derived(profileProblemText(rule));
  let ending = $derived(endingProfileText(rule.trigger, rule.action));
  let lines = $derived(rule.last_run ? runLines(rule.last_run) : []);
  let failed = $derived(rule.last_run ? runFailed(rule.last_run) : false);
  let open = $state(false);

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

<li class="rule" {id}>
  <div class="row" role="group" aria-label={sentence(rule)}>
    <div class="head">
      <Switch
        {label}
        badge={false}
        on={rule.enabled}
        disabled={locked}
        describedby="{id}-promise"
        onchange={(on) => automation.setRule(rule.trigger, on, rule.action)}
      />
      <div class="selects">
        <select value={kind} disabled={locked} aria-label="Action {event}" onchange={chooseAction}>
          <option value="Nothing">Do nothing</option>
          <option value="LightsOff">Switch the lights off</option>
          <option value="ApplyProfile">Apply a profile</option>
          {#if stored === "Newer"}
            <option value="Newer" disabled>Set by a newer PrismForge</option>
          {/if}
        </select>
        {#if kind === "ApplyProfile"}
          <select
            value={known && chosen === null ? profile : ""}
            disabled={locked || profiles.length === 0}
            aria-label="Profile {event}"
            onchange={chooseProfile}
          >
            <option value="" disabled>{profiles.length === 0 ? "No profiles saved yet" : "Choose a profile"}</option>
            {#each profiles as choice (choice.id)}
              <option value={choice.id}>{choice.name}</option>
            {/each}
          </select>
        {/if}
      </div>
    </div>

    <div class="body">
      <p class="promise">
        <span id="{id}-promise">{shortPromiseText(rule.trigger, rule.action)}</span>
        <button type="button" class="quiet more" aria-expanded={open} aria-controls="{id}-details" onclick={() => (open = !open)}>
          Details<span class="visually-hidden"> {event}</span>
        </button>
      </p>
      <div id="{id}-details" class="details" data-details hidden={!open}>
        <p>{promiseText(rule.trigger, rule.action)}</p>
        {#if ending}
          <p>{ending}</p>
        {/if}
        {#if kind === "LightsOff"}
          <p>{LIGHTS_OFF}</p>
        {/if}
        {#each rule.openrgb_also as act (act.setting)}
          <p>{openRgbAlsoText(act)}</p>
        {/each}
      </div>
      {#if problem}
        <p class="notice"><ToneIcon tone="problem" /><span>{problem}</span></p>
      {/if}
      {#each rule.openrgb_also as act (act.setting)}
        <p class="notice"><ToneIcon tone="warning" /><span>{openRgbAlsoShortText(act)}</span></p>
      {/each}
      {#if failure}
        <p class="notice" role="alert"><ToneIcon tone="problem" /><span>{failure}</span></p>
      {/if}
      {#if rule.last_run}
        <p class="last" class:failed>
          {#if failed}<ToneIcon tone="problem" />{/if}
          <span>Last run {runTime(rule.last_run)} · {lines[0]}</span>
        </p>
        {#each lines.slice(1) as line, index (index)}
          <p class="extra">{line}</p>
        {/each}
      {/if}
    </div>
  </div>
</li>

<style>
  .rule {
    padding: var(--space-3) 0;
    border-top: 1px solid var(--line-subtle);
  }

  .row {
    display: grid;
    gap: var(--space-1);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
  }

  .selects {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-left: auto;
  }

  .selects select {
    min-width: 12rem;
    max-width: 100%;
  }

  .body {
    display: grid;
    gap: var(--space-1);
    padding-left: calc(34px + var(--space-2));
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .body p {
    max-width: 72ch;
  }

  .head :global(button[role="switch"]:disabled) {
    border-color: transparent;
  }

  .promise {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    column-gap: var(--space-2);
  }

  .more {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-1);
    font-size: var(--font-s);
    color: var(--text-dim);
  }

  .more::after {
    content: "";
    width: 6px;
    height: 6px;
    border-right: 1.5px solid currentColor;
    border-bottom: 1.5px solid currentColor;
    transform: translateY(-2px) rotate(45deg);
  }

  .more[aria-expanded="true"]::after {
    transform: translateY(2px) rotate(-135deg);
  }

  .more:hover {
    color: var(--text);
  }

  .details {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-1) 0 var(--space-2) var(--space-3);
    border-left: 1px solid var(--line-subtle);
  }

  .details[hidden] {
    display: none;
  }

  .notice,
  .last {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }

  .notice,
  .failed {
    color: var(--text);
  }

  @media (max-width: 700px) {
    .selects {
      margin-left: calc(34px + var(--space-2));
    }
  }
</style>
