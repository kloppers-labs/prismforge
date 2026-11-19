<section class="needs" aria-labelledby="needs-title">
  <h3 id="needs-title">Before you connect</h3>
  <ul>
    <li>OpenRGB 1.0 or later is running.</li>
    <li>Its SDK server is on. In OpenRGB, open the SDK Server tab and select Start Server, or start OpenRGB with the <code>--server</code> option.</li>
    <li>The port here matches the port shown in that tab.</li>
  </ul>
  <p>PrismForge does not download, install or change OpenRGB. It writes nothing to your devices until you choose an action such as Identify, Apply effect or Apply profile.</p>
</section>

<style>
  .needs {
    display: grid;
    gap: 8px;
    padding-top: 16px;
    border-top: 1px solid var(--line);
  }

  ul {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 4px;
    color: var(--text-dim);
  }

  p {
    color: var(--text-faint);
    font-size: 13px;
  }

  code {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 13px;
    color: var(--text);
  }
</style>
