<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import Prerequisites from "./Prerequisites.svelte";
  import { isSimulated, summarise } from "./wording";

  let { session }: { session: SessionSnapshot } = $props();

  let summary = $derived(summarise(session));
  let live = $derived(summary.tone === "ok" || summary.tone === "warning");
</script>

<section class="waiting" aria-labelledby="waiting-title">
  <div class="head">
    <ToneIcon tone={summary.tone} />
    <h1 id="waiting-title">
      {#if live}
        No devices found
      {:else if summary.retrying}
        Waiting for OpenRGB
      {:else}
        Connecting
      {/if}
    </h1>
  </div>
  {#each summary.reasons as reason (reason)}
    <p>{reason}</p>
  {/each}
  {#if live}
    <p>OpenRGB answered but reports no devices. Check the device list in OpenRGB, then select Refresh devices.</p>
  {:else if !summary.retrying}
    <p class="loading" aria-busy="true">Reading the device list when the server answers.</p>
  {/if}
  {#if !isSimulated(session) && !live}
    <Prerequisites />
  {/if}
</section>

<style>
  .waiting {
    max-width: 640px;
    margin: 0 auto;
    padding: 64px 32px;
    display: grid;
    gap: 12px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  p {
    color: var(--text-dim);
    max-width: 62ch;
  }

  .loading::after {
    content: "";
    display: block;
    height: 2px;
    margin-top: 12px;
    background: var(--spectrum);
    background-size: 200% 100%;
    animation: sweep 1.6s linear infinite;
    border-radius: 1px;
  }

  @keyframes sweep {
    from {
      background-position: 0 0;
    }
    to {
      background-position: 200% 0;
    }
  }
</style>
