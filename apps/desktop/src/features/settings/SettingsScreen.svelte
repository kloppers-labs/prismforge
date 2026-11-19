<script lang="ts">
  import { onMount } from "svelte";
  import type { CloseAction } from "../../lib/api/bindings/CloseAction";
  import type { StartEntryState } from "../../lib/api/bindings/StartEntryState";
  import { failureText, storeUnavailableText } from "../../lib/errors";
  import Switch from "../../lib/Switch.svelte";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import type { SettingKey, LifecycleState } from "./state.svelte";
  import { addressText, quitShortcut, settingsNoticeText, startPathText, trayName } from "./wording";

  let { lifecycle, onprofiles }: { lifecycle: LifecycleState; onprofiles: () => void } = $props();

  onMount(() => {
    lifecycle.load();
  });

  let view = $derived(lifecycle.view);
  let settings = $derived(lifecycle.settings);
  let tray = $derived(lifecycle.trayAvailable);
  let ready = $derived(settings !== null);
  let unavailable = $derived(view && "Unavailable" in view.settings ? view.settings.Unavailable.reason : null);
  let notices = $derived(view && "Ready" in view.settings ? view.settings.Ready.notices : []);
  let entry = $derived<StartEntryState>(view?.start_with_system ?? "NotRegistered");
  let startProfile = $derived(view?.start_profile ?? null);
  let startPath = $derived(
    startPathText({
      system: entry === "Registered",
      hidden: settings?.start_hidden ?? false,
      connect: settings?.connect_at_start ?? false,
      address: settings?.address ? addressText(settings.address) : null,
      profile: startProfile !== null && typeof startProfile === "object" && "Marked" in startProfile ? startProfile.Marked.name : null,
    }),
  );
  let trayDetail = $derived(view && typeof view.tray === "object" ? view.tray.Unavailable.detail : null);

  function failed(key: SettingKey | "quit"): string | null {
    const failure = lifecycle.failure;
    return failure && failure.key === key ? failureText(failure.error) : null;
  }

  function choose(action: CloseAction) {
    if (settings?.on_close !== action) {
      lifecycle.change({ OnClose: action });
    }
  }

  function closeKey(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp" || event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      const next: CloseAction = settings?.on_close === "Background" ? "Quit" : "Background";
      choose(next);
      document.getElementById(next === "Background" ? "close-background" : "close-quit")?.focus();
    }
  }
</script>

<div class="settings">
  <header class="intro">
    <h1>Settings</h1>
    <p>These choices are stored on this computer. They apply to OpenRGB and demo mode alike.</p>
  </header>

  {#if unavailable}
    <div class="notice problem" role="alert">
      <ToneIcon tone="problem" />
      <p>{storeUnavailableText(unavailable)} The settings below cannot be changed now, and every one of them counts as off.</p>
    </div>
  {/if}

  {#each notices as notice, index (index)}
    <div class="notice warning" role="status">
      <ToneIcon tone="warning" />
      <div>
        {#each settingsNoticeText(notice) as line (line)}
          <p>{line}</p>
        {/each}
      </div>
    </div>
  {/each}

  <section class="group" aria-labelledby="close-title">
    <h2 id="close-title">When you close the window</h2>
    <div class="choices" role="radiogroup" aria-labelledby="close-title" aria-describedby="close-help">
      <button
        id="close-background"
        type="button"
        role="radio"
        aria-checked={settings?.on_close === "Background" ? "true" : "false"}
        tabindex={settings?.on_close === "Quit" ? -1 : 0}
        disabled={!ready || !tray || lifecycle.pending !== null}
        onclick={() => choose("Background")}
        onkeydown={closeKey}
      >
        <span class="dot" aria-hidden="true"></span>
        Keep PrismForge running in {trayName()}
      </button>
      <button
        id="close-quit"
        type="button"
        role="radio"
        aria-checked={settings?.on_close === "Quit" ? "true" : "false"}
        tabindex={settings?.on_close === "Quit" ? 0 : -1}
        disabled={!ready || lifecycle.pending !== null}
        onclick={() => choose("Quit")}
        onkeydown={closeKey}
      >
        <span class="dot" aria-hidden="true"></span>
        Quit PrismForge
      </button>
    </div>
    <div id="close-help" class="help">
      {#if !tray}
        <p class="unavailable">
          Running in the background is not available: this system has no tray{trayDetail ? ` (${trayDetail})` : ""}. Closing the window quits PrismForge, and its effects stop.
        </p>
      {:else}
        <p>In {trayName()}, PrismForge keeps running without its window, and its effects keep running on your devices. Open the window again from its icon there.</p>
        <p>To quit completely, choose Quit PrismForge in that menu or at the bottom of this screen.</p>
        {#if ready && settings?.on_close === null}
          <p class="pending">Not chosen yet. PrismForge asks the first time you close the window.</p>
        {/if}
      {/if}
    </div>
    {#if failed("OnClose")}
      <p class="failure" role="alert">{failed("OnClose")}</p>
    {/if}
  </section>

  <section class="group" aria-labelledby="start-title">
    <h2 id="start-title">When PrismForge starts</h2>
    <p class="help">
      With steps 1, 3 and 4 on, your lights change at system start without a click. Every step is off until you turn it on.
    </p>

    <ol class="steps" aria-label="Start-up steps">
      <li>
        <Switch
          label="Start PrismForge when you sign in"
          on={entry === "Registered"}
          disabled={lifecycle.pending !== null}
          describedby="system-help"
          onchange={(on) => lifecycle.change({ StartWithSystem: on })}
        />
        <div id="system-help" class="help">
          <p>Adds PrismForge to the programs your system starts for your user account. It needs no administrator rights. Turn it off before you uninstall PrismForge.</p>
          {#if typeof entry === "object"}
            <p class="unavailable">PrismForge cannot read its start entry: {entry.Unknown.detail}</p>
          {/if}
        </div>
        {#if failed("StartWithSystem")}
          <p class="failure" role="alert">{failed("StartWithSystem")}</p>
        {/if}
      </li>

      <li>
        <Switch
          label="Start hidden in {trayName()}"
          on={settings?.start_hidden ?? false}
          disabled={!ready || !tray || lifecycle.pending !== null}
          describedby="hidden-help"
          onchange={(on) => lifecycle.change({ StartHidden: on })}
        />
        <div id="hidden-help" class="help">
          {#if tray}
            <p>Optional. Only when your system starts PrismForge: the window stays closed and PrismForge waits in {trayName()}. When you start PrismForge yourself, the window opens.</p>
          {:else}
            <p class="unavailable">Not available: this system has no tray, so the window always opens.</p>
          {/if}
        </div>
        {#if failed("StartHidden")}
          <p class="failure" role="alert">{failed("StartHidden")}</p>
        {/if}
      </li>

      <li>
        <Switch
          label="Connect to OpenRGB when PrismForge starts"
          on={settings?.connect_at_start ?? false}
          disabled={!ready || lifecycle.pending !== null || (!settings?.address && !settings?.connect_at_start)}
          describedby="connect-help"
          onchange={(on) => lifecycle.change({ ConnectAtStart: on })}
        />
        <div id="connect-help" class="help">
          {#if settings?.address}
            <p>PrismForge connects to <span class="address">{addressText(settings.address)}</span>, the address of your last Connect. If OpenRGB does not answer, it keeps retrying, as after Connect. It never starts demo mode.</p>
          {:else}
            <p>Connect to OpenRGB once first. PrismForge then remembers that address.</p>
          {/if}
        </div>
        {#if failed("ConnectAtStart")}
          <p class="failure" role="alert">{failed("ConnectAtStart")}</p>
        {/if}
      </li>

      <li>
        <p class="step-title">Apply the profile marked “Apply this profile when PrismForge connects”</p>
        <div class="profile">
          {#if startProfile === null || startProfile === "NotMarked"}
            <p>None marked.</p>
          {:else if "Marked" in startProfile}
            <p>Applies <strong>{startProfile.Marked.name}</strong>, once, as soon as the connection is made.</p>
          {:else if "Unreadable" in startProfile}
            <p class="unavailable">
              {startProfile.Unreadable.problem === "NewerVersion"
                ? "The marked profile needs a newer PrismForge, so nothing is applied."
                : "The marked profile is no longer in the store, so nothing is applied."}
            </p>
          {:else}
            <p class="unavailable">{storeUnavailableText(startProfile.Unavailable.reason)}</p>
          {/if}
          <button type="button" class="small" onclick={onprofiles}>Open Profiles</button>
        </div>
        <p class="help">Mark one of your OpenRGB profiles on the Profiles screen. Demo profiles are never applied at start.</p>
      </li>
    </ol>

    <p class="result" role="status" aria-label="Start-up result">{startPath}</p>

    <aside class="automatic" aria-labelledby="automatic-title">
      <h3 id="automatic-title">When your lighting changes without a click</h3>
      <p>
        Connecting alone writes nothing to your devices. Without a click in a session, PrismForge changes your lighting only when “Connect to OpenRGB when PrismForge starts” is on and a profile is set to “Apply this profile when PrismForge connects”. Then PrismForge applies that profile as soon as the connection is made.
      </p>
      <p>PrismForge effects run only while PrismForge runs and is connected. Nothing is stored on your devices.</p>
    </aside>
  </section>

  <section class="group" aria-labelledby="files-title">
    <h2 id="files-title">Where PrismForge keeps its files</h2>
    <p class="help">Your profiles, your names for devices and these settings are in this folder on this computer:</p>
    <p class="folder">{view?.folder ?? ""}</p>
    <p class="help">The frame rate of PrismForge effects is on the Effects screen while PrismForge is connected.</p>
  </section>

  <section class="group quit" aria-labelledby="quit-title">
    <h2 id="quit-title">Quit PrismForge</h2>
    <p class="help">
      Stops PrismForge effects, puts back the lighting from before where it can, disconnects and ends PrismForge. The shortcut is {quitShortcut()}.
    </p>
    <button type="button" disabled={lifecycle.pending === "quit"} onclick={() => lifecycle.quit()}>
      {lifecycle.pending === "quit" ? "Quitting PrismForge" : "Quit PrismForge"}
    </button>
    {#if failed("quit")}
      <p class="failure" role="alert">{failed("quit")}</p>
    {/if}
  </section>
</div>

<style>
  .settings {
    max-width: 780px;
    margin: 0 auto;
    padding: 40px 32px 56px;
    display: grid;
    gap: 28px;
  }

  .intro {
    display: grid;
    gap: 6px;
  }

  .intro p,
  .help {
    color: var(--text-dim);
  }

  .help {
    display: grid;
    gap: 4px;
    font-size: 13px;
    max-width: 68ch;
  }

  .group {
    display: grid;
    gap: 14px;
    padding-top: 22px;
    border-top: 1px solid var(--line);
  }

  .group h2 {
    font-size: 16px;
  }

  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 14px;
    counter-reset: step;
  }

  .steps > li {
    counter-increment: step;
    display: grid;
    gap: 4px;
    padding-left: 38px;
    position: relative;
  }

  .steps > li::before {
    content: counter(step);
    position: absolute;
    left: 0;
    top: 4px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid var(--line);
    display: grid;
    place-items: center;
    font-size: 12px;
    color: var(--text-dim);
    background: var(--ink-1);
  }

  .step-title {
    font-weight: 600;
    padding: 6px 0 0;
  }

  .profile {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    font-size: 13px;
  }

  .profile strong {
    color: var(--text);
  }

  .small {
    padding: 2px 10px;
    font-size: 13px;
  }

  .result {
    font-size: 13px;
    max-width: 72ch;
    padding: 10px 12px;
    border-radius: var(--radius-s);
    border: 1px solid var(--line);
    background: var(--ink-0);
  }

  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  [role="radio"] {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
  }

  .dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid var(--text-faint);
    flex: none;
  }

  [role="radio"][aria-checked="true"] {
    border-color: var(--ok);
    background: var(--ink-2);
  }

  [role="radio"][aria-checked="true"] .dot {
    border-color: var(--ok);
    background: radial-gradient(circle, var(--ok) 0 40%, transparent 45%);
  }

  .pending {
    color: var(--warning);
  }

  .unavailable {
    color: var(--text);
  }

  .failure {
    color: var(--problem);
    font-size: 13px;
    max-width: 68ch;
  }

  .automatic {
    display: grid;
    gap: 8px;
    padding: 14px 16px 14px 18px;
    border-radius: var(--radius-s);
    background: var(--ink-1);
    border: 1px solid var(--line);
    position: relative;
    max-width: 72ch;
  }

  .automatic::before {
    content: "";
    position: absolute;
    left: -1px;
    top: -1px;
    bottom: -1px;
    width: 3px;
    border-radius: var(--radius-s) 0 0 var(--radius-s);
    background: linear-gradient(180deg, #ff5a4e, #ffb340, #f4e04d, #4fd18b, #3fa9ff, #9b7bff);
  }

  .automatic h3 {
    font-size: 14px;
  }

  .automatic p {
    color: var(--text-dim);
    font-size: 13px;
  }

  .address,
  .folder {
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .folder {
    user-select: text;
    overflow-wrap: anywhere;
    padding: 8px 10px;
    border-radius: var(--radius-s);
    background: var(--ink-0);
    border: 1px solid var(--line);
    font-size: 13px;
  }

  .quit button {
    justify-self: start;
  }

  .notice {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 10px 14px;
    border-radius: var(--radius-s);
    border: 1px solid var(--line);
    background: var(--ink-1);
  }

  .notice :global(svg) {
    margin-top: 3px;
  }

  .notice.problem {
    background: #2a1d1f;
  }

  .notice.warning {
    background: #2a2519;
  }
</style>
