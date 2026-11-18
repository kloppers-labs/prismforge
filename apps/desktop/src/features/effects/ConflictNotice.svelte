<script lang="ts">
  let { label }: { label: string } = $props();
</script>

<p class="headline">You said {label} controls this device. PrismForge sends nothing to it until you hand it back.</p>
<p>First stop this device's effect in {label}. Then choose Hand back to PrismForge below. PrismForge cannot see or stop other programs.</p>
<p>If two programs write to one device, the lights flicker or the last writer wins.</p>

<style>
  .headline {
    color: var(--text);
    font-weight: 600;
  }
</style>
