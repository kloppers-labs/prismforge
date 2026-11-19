<script lang="ts">
  import type { SessionDevice } from "../../lib/api/bindings/SessionDevice";
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import { reference } from "../devices/present";
  import { outputOf, ownerText } from "./present";

  let { session, device, zone }: { session: SessionSnapshot; device: SessionDevice; zone: number | null } = $props();

  let ref = $derived(reference(device));
  let output = $derived(ref === null ? null : outputOf(session, ref));
</script>

<dl class="status">
  <dt>Controlled by</dt>
  <dd>
    {ownerText(device, output)}
    {#if zone !== null}<span class="note">This belongs to the whole device: a hardware mode or another program affects every zone.</span>{/if}
  </dd>
</dl>

<style>
  .status {
    display: grid;
    grid-template-columns: minmax(0, 8rem) minmax(0, 1fr);
    gap: var(--space-1) var(--space-4);
    margin: 0;
  }

  dt {
    color: var(--text-dim);
  }

  dd {
    margin: 0;
    max-width: 72ch;
  }

  .note {
    display: block;
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
