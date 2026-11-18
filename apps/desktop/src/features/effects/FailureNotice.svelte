<script lang="ts">
  import type { Failure } from "../../lib/api/client";
  import { failureText } from "../../lib/errors";
  import ConflictNotice from "./ConflictNotice.svelte";

  let { error }: { error: Failure } = $props();

  let conflict = $derived.by(() => {
    if (typeof error !== "object" || !("Session" in error)) {
      return null;
    }
    const session = error.Session;
    if (typeof session !== "object" || !("OutputConflict" in session)) {
      return null;
    }
    const owner = session.OutputConflict.owner;
    return typeof owner === "object" && "External" in owner ? owner.External.label : "another program";
  });
</script>

<div class="failure" role="alert">
  {#if conflict !== null}
    <ConflictNotice label={conflict} />
  {:else}
    <p>{failureText(error)}</p>
  {/if}
</div>

<style>
  .failure {
    display: grid;
    gap: 4px;
    padding: 10px 14px;
    border-left: 3px solid var(--warning);
    background: var(--ink-0);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
    color: var(--text-dim);
  }
</style>
