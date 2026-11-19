<script lang="ts">
  let { name, unsaved, onopen }: { name: string; unsaved: number | null; onopen: () => void } = $props();

  let changes = $derived(
    unsaved === null ? "Unsaved changes unknown" : unsaved === 0 ? null : unsaved === 1 ? "1 unsaved change" : `${unsaved} unsaved changes`,
  );
</script>

<button type="button" class="status" onclick={onopen}>
  Profile <strong>{name}</strong>
  {#if changes}<span class="changes">{changes}</span>{/if}
</button>

<style>
  .status {
    display: inline-flex;
    gap: 6px;
    align-items: baseline;
    padding: 4px 12px;
    font-weight: 450;
    color: var(--text-dim);
    white-space: nowrap;
  }

  strong {
    color: var(--text);
    font-weight: 600;
    max-width: 22ch;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .changes {
    font-size: 12px;
    padding: 0 6px;
    border: 1px solid var(--warning);
    border-radius: 4px;
    color: var(--text);
  }
</style>
