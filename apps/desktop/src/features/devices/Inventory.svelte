<script lang="ts">
  import type { SessionSnapshot } from "../../lib/api/bindings/SessionSnapshot";
  import type { AppState } from "../../lib/state/app.svelte";
  import DeviceDetail from "./DeviceDetail.svelte";
  import DeviceList from "./DeviceList.svelte";
  import { deviceKey } from "./present";

  let { app, session }: { app: AppState; session: SessionSnapshot } = $props();

  let entries = $derived(session.devices.map((device, index) => ({ key: deviceKey(device, index), device })));
  let current = $derived(entries.find((entry) => entry.key === app.selected) ?? entries[0]);
</script>

<div class="inventory">
  <DeviceList {entries} selected={current?.key ?? null} issues={session.issues.length} names={app.names} onselect={(key) => (app.selected = key)} />
  {#if current}
    {#key current.key}
      <DeviceDetail {app} {session} device={current.device} />
    {/key}
  {/if}
</div>

<style>
  .inventory {
    display: grid;
    grid-template-columns: minmax(260px, 320px) 1fr;
    min-height: 0;
    height: 100%;
  }
</style>
