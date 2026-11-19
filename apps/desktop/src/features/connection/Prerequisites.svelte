<script lang="ts">
  import Details from "../../lib/Details.svelte";
</script>

<div class="needs">
  <p class="line">You need OpenRGB 1.0 or later with its SDK server on.</p>
  <Details summary="What you need, and what PrismForge writes">
    <ul>
      <li>OpenRGB 1.0 or later is running.</li>
      <li>Its SDK server is on. In OpenRGB, open the SDK Server tab and select Start Server, or start OpenRGB with the <code>--server</code> option.</li>
      <li>The port here matches the port shown in that tab.</li>
    </ul>
    <p>PrismForge does not download, install or change OpenRGB. It writes nothing to your devices until you choose an action such as Identify, Apply effect or Apply profile, or until it connects while a profile is set to “Apply this profile when PrismForge connects”: that profile is applied as soon as the connection is made.</p>
  </Details>
</div>

<style>
  .needs {
    display: grid;
    gap: var(--space-1);
  }

  .line {
    color: var(--text-dim);
    font-size: var(--font-s);
  }

  ul {
    margin: 0;
    padding-left: var(--space-4);
    display: grid;
    gap: var(--space-1);
  }

  code {
    font-family: var(--font-mono);
    color: var(--text);
    white-space: nowrap;
  }
</style>
