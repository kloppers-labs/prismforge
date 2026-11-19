<script lang="ts">
  import type { StoreNotice } from "../../lib/api/bindings/StoreNotice";
  import ToneIcon from "../../lib/ToneIcon.svelte";
  import { noticeText } from "./present";

  let { notices, ondismiss }: { notices: StoreNotice[]; ondismiss: (notice: StoreNotice) => void } = $props();
</script>

<section class="notices" aria-labelledby="notices-title">
  <h3 id="notices-title">Repaired files</h3>
  <ul>
    {#each notices as notice, index (index)}
      <li>
        <ToneIcon tone="warning" />
        <div>
          {#each noticeText(notice) as line (line)}
            <p>{line}</p>
          {/each}
        </div>
        <button type="button" class="small" aria-label="Dismiss the notice about {notice.file}" onclick={() => ondismiss(notice)}>Dismiss</button>
      </li>
    {/each}
  </ul>
  <p class="hint">Diagnostics shows the folder these files are in. A file whose name ends in .tmp is safe to delete while PrismForge is closed.</p>
</section>

<style>
  .notices {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-left: 3px solid var(--warning);
    background: var(--surface-1);
    border-radius: 0 var(--radius-s) var(--radius-s) 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-3);
  }

  li {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-2);
    align-items: start;
  }

  li :global(svg) {
    margin-top: var(--space-1);
  }

  li p {
    max-width: 72ch;
  }

  .small {
    padding: 3px var(--space-2);
    font-size: var(--font-s);
  }

  .hint {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
