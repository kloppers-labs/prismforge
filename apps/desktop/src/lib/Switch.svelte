<script lang="ts">
  let {
    label,
    on,
    disabled = false,
    describedby,
    onchange,
  }: { label: string; on: boolean; disabled?: boolean; describedby: string; onchange: (on: boolean) => void } = $props();
</script>

<button type="button" role="switch" aria-checked={on ? "true" : "false"} aria-describedby={describedby} {disabled} onclick={() => onchange(!on)}>
  <span class="track" aria-hidden="true"><span class="thumb"></span></span>
  <span class="label">{label}</span>
  <span class="state" aria-hidden="true">{on ? "On" : "Off"}</span>
</button>

<style>
  button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    justify-self: start;
    background: transparent;
    border-color: transparent;
    padding: 6px 8px 6px 0;
    text-align: left;
  }

  button:disabled {
    opacity: 0.55;
  }

  .label {
    font-weight: var(--weight-strong);
  }

  .track {
    width: 34px;
    height: 20px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--line-control);
    background: var(--surface-0);
    position: relative;
    flex: none;
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: transform var(--dur-fast) var(--ease-out);
  }

  [aria-checked="true"] .track {
    background: var(--surface-3);
    border-color: var(--ok);
  }

  [aria-checked="true"] .thumb {
    transform: translateX(14px);
    background: var(--ok);
  }

  .state {
    font-size: var(--font-xs);
    color: var(--text-dim);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-xs);
    padding: 0 6px;
  }

  @media (forced-colors: active) {
    .thumb {
      forced-color-adjust: none;
      background: CanvasText;
    }

    [aria-checked="true"] .thumb {
      background: Highlight;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .thumb {
      transition: none;
    }
  }
</style>
