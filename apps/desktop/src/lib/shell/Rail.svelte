<script lang="ts">
  import Mark from "../Mark.svelte";
  import type { Screen } from "../state/app.svelte";
  import { MAIN_AREAS, SETTINGS_AREA, type Area } from "./areas";
  import Icon from "./Icon.svelte";
  import { shortcutKeys, shortcutText } from "./keyboard";

  let {
    screen,
    diagnosticsOpen,
    ongo,
    ondiagnostics,
    region = $bindable(),
    diagnosticsButton = $bindable(),
  }: {
    screen: Screen;
    diagnosticsOpen: boolean;
    ongo: (screen: Screen) => void;
    ondiagnostics: () => void;
    region?: HTMLElement;
    diagnosticsButton?: HTMLButtonElement;
  } = $props();
</script>

{#snippet item(area: Area)}
  <li>
    <button
      type="button"
      class="item"
      aria-current={screen === area.screen ? "page" : undefined}
      aria-keyshortcuts={shortcutKeys(area)}
      title="{area.label} ({shortcutText(area)})"
      onclick={() => ongo(area.screen)}
    >
      <Icon name={area.icon} />
      <span class="label">{area.label}</span>
    </button>
  </li>
{/snippet}

<nav class="rail" aria-label="Main" bind:this={region}>
  <div class="brand">
    <Mark />
    <span class="name">PrismForge</span>
  </div>
  <ul class="group">
    {#each MAIN_AREAS as area (area.screen)}
      {@render item(area)}
    {/each}
  </ul>
  <ul class="group foot">
    <li>
      <button
        type="button"
        class="item"
        bind:this={diagnosticsButton}
        aria-expanded={diagnosticsOpen}
        aria-controls="diagnostics-drawer"
        onclick={ondiagnostics}
      >
        <Icon name="diagnostics" />
        <span class="label">Diagnostics</span>
      </button>
    </li>
    {@render item(SETTINGS_AREA)}
  </ul>
</nav>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: var(--rail-w);
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    scrollbar-width: thin;
    padding: var(--space-3) var(--space-2);
    background: var(--surface-1);
    border-right: 1px solid var(--line-subtle);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2) var(--space-3);
  }

  .name {
    font-weight: var(--weight-heading);
    letter-spacing: 0.01em;
  }

  .group {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  .foot {
    margin-top: auto;
    padding-top: var(--space-2);
    border-top: 1px solid var(--line-subtle);
  }

  .item {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    min-height: var(--control-h);
    padding: var(--space-2) var(--space-3);
    border: 0;
    background: transparent;
    color: var(--text-dim);
    font-weight: var(--weight-regular);
    text-align: left;
    --icon-knob: var(--surface-1);
  }

  .item:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
    --icon-knob: var(--surface-2);
  }

  .item[aria-current="page"],
  .item[aria-expanded="true"] {
    background: var(--surface-3);
    color: var(--text);
    --icon-knob: var(--surface-3);
  }

  .item[aria-current="page"] {
    font-weight: var(--weight-heading);
  }

  .item[aria-current="page"]::before {
    content: "";
    position: absolute;
    left: calc(-1 * var(--space-2));
    top: var(--space-1);
    bottom: var(--space-1);
    width: 3px;
    border-radius: 0 var(--radius-xs) var(--radius-xs) 0;
    background: var(--accent);
  }

  .label {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  @media (max-width: 1099px) {
    .rail {
      width: var(--rail-w-compact);
      padding: var(--space-3) var(--space-1);
    }

    .brand {
      justify-content: center;
      padding: var(--space-1) 0 var(--space-3);
    }

    .name {
      display: none;
    }

    .item {
      flex-direction: column;
      gap: 2px;
      padding: var(--space-2) 2px;
      text-align: center;
    }

    .item[aria-current="page"]::before {
      left: calc(-1 * var(--space-1));
    }

    .label {
      font-size: var(--font-2xs);
      line-height: var(--leading-tight);
      overflow-wrap: normal;
    }
  }
</style>
