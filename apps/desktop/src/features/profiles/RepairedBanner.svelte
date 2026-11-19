<script lang="ts">
  import ToneIcon from "../../lib/ToneIcon.svelte";

  let { onopen }: { onopen: () => void } = $props();
</script>

<div class="banner" role="status" aria-label="Repaired files">
  <ToneIcon tone="warning" />
  <p>PrismForge repaired stored files. Open Profiles to read what happened and what to check.</p>
  <button type="button" class="small" onclick={onopen}>Open Profiles</button>
</div>

<style>
  .banner {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 20px;
    border-bottom: 1px solid var(--line);
    background: #2a2519;
  }

  p {
    flex: 1;
  }

  .small {
    padding: 3px 10px;
    font-size: 13px;
  }
</style>
