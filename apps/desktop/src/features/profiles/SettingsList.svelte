<script lang="ts">
  import type { ProfileDetails } from "../../lib/api/bindings/ProfileDetails";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import { deviceLabel, itemText, skipText, type Labels } from "./present";

  let { details, labels }: { details: ProfileDetails; labels: Labels } = $props();

  function targetOf(reference: string): string | null {
    const status = details.devices.find((device) => device.reference === reference)?.status;
    if (status === undefined || typeof status === "string") {
      return null;
    }
    return "Resolved" in status ? status.Resolved.device : status.Unavailable.device;
  }
</script>

<section class="settings" aria-labelledby="settings-title">
  <h3 id="settings-title">What this profile holds</h3>
  <p class="hint">Frame rate {details.frame_rate} frames per second.</p>
  {#if !details.checked}
    <p class="hint">Not checked: PrismForge is not connected. Connect to see which settings would apply today.</p>
  {/if}
  {#if details.settings.length === 0}
    <p class="hint">This profile holds no effect, hardware mode or program statement.</p>
  {:else}
    <ul aria-label="What this profile holds">
      {#each details.settings as setting, index (index)}
        {@const target = targetOf(setting.device)}
        <li>
          <span class="what">
            <strong>{deviceLabel(labels, target, setting.label)}</strong>
            <span>{itemText(labels, setting.item, target, { zone: setting.zone_name, effect: setting.effect_label })}</span>
          </span>
          {#if details.checked}
            {#if setting.today === "Ready"}
              <span class="today"><ToneIcon tone="ok" /> Ready today.</span>
            {:else}
              <span class="today skipped"><ToneIcon tone="warning" /> Skipped today. {skipText(labels, setting.today.Skipped.reason)}</span>
            {/if}
          {/if}
        </li>
      {/each}
    </ul>
    <p class="hint">Unresolved settings stay in the profile. PrismForge never moves them to a similar device.</p>
  {/if}
</section>

<style>
  .settings {
    display: grid;
    gap: 10px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 4px 20px;
    padding: 9px 0;
    border-bottom: 1px solid var(--ink-3);
  }

  .what {
    display: grid;
  }

  .what span {
    color: var(--text-dim);
  }

  .today {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: var(--text-dim);
  }

  .today :global(svg) {
    flex: none;
    margin-top: 4px;
  }

  .skipped {
    color: var(--text);
  }

  .hint {
    color: var(--text-faint);
    font-size: 13px;
  }
</style>
