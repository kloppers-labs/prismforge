<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { Names } from "../../lib/names";
  import { kindLabel, ledCount, reference } from "../devices/present";
  import { conditionText, outputOf, ownerText, targetLeds, targetName } from "./present";

  let {
    session,
    device,
    zone,
    names,
  }: { session: SessionSnapshot; device: SessionDevice; zone: number | null; names: Names } = $props();

  let ref = $derived(reference(device));
  let output = $derived(ref === null ? null : outputOf(session, ref));
  let condition = $derived(conditionText(session, output));
</script>

<header class="status">
  <h2 id="target-title">{names.device(device)}</h2>
  <p class="sub">
    {zone === null ? `Whole device, ${kindLabel(device.descriptor.kind).toLowerCase()}` : `Zone ${targetName(names, device, zone)}`},
    {ledCount(targetLeds(device, zone))}
  </p>
  <dl>
    <dt>Owner</dt>
    <dd>{ownerText(device, output)}</dd>
    {#if condition}
      <dt>Output</dt>
      <dd>{condition}</dd>
    {/if}
  </dl>
  {#if zone !== null}
    <p class="note">The owner belongs to the whole device. A hardware mode or another program affects every zone.</p>
  {/if}
</header>

<style>
  .status {
    display: grid;
    gap: 6px;
  }

  .sub {
    color: var(--text-dim);
  }

  dl {
    display: grid;
    grid-template-columns: 80px 1fr;
    gap: 4px 16px;
    margin: 8px 0 0;
  }

  dt {
    color: var(--text-faint);
  }

  dd {
    margin: 0;
    max-width: 72ch;
  }

  .note {
    color: var(--text-faint);
    font-size: var(--font-s);
  }
</style>
