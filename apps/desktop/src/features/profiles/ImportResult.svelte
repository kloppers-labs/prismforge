<script lang="ts">
  import type { Failure } from "../../lib/api/client";
  import type { ImportReport } from "../../lib/api/bindings/ImportReport";
  import { failureText } from "../../lib/errors";

  let { report, problem }: { report: ImportReport | null; problem: Failure | null } = $props();

  let details = $derived(report?.details ?? null);
  let resolved = $derived(details ? details.devices.filter((device) => typeof device.status === "object" && "Resolved" in device.status).length : 0);
  let unsupported = $derived(details?.unsupported.length ?? 0);
</script>

<div class="import" role="status" aria-label="Import result">
  {#if problem}
    <p class="problem">{failureText(problem)}</p>
  {:else if report && details}
    <p class="headline">Imported as {details.name}.</p>
    <p>
      {report.renamed_from
        ? `Another profile was already called ${report.renamed_from}, so the import got a new name. Nothing was replaced.`
        : "Nothing was replaced."}
    </p>
    {#if details.checked}
      <p>
        {resolved === details.devices.length
          ? `All ${details.devices.length} devices found today.`
          : `${resolved} of ${details.devices.length} devices found today. The others stay in the profile, unresolved.`}
      </p>
    {:else}
      <p>Devices not checked: PrismForge is not connected.</p>
    {/if}
    {#if unsupported > 0}
      <p>
        {unsupported === 1
          ? "1 effect is not available in this version. It stays in the profile."
          : `${unsupported} effects are not available in this version. They stay in the profile.`}
      </p>
    {/if}
    <p>Importing writes nothing to your devices.</p>
  {/if}
</div>

<style>
  .import {
    display: grid;
    gap: 4px;
  }

  .headline {
    font-weight: var(--weight-strong);
  }

  .problem {
    color: var(--problem);
  }
</style>
