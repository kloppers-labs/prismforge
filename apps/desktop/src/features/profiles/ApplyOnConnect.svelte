<script lang="ts">
  let { on, onchange }: { on: boolean; onchange: (on: boolean) => void } = $props();

  const id = `on-connect-${Math.random().toString(36).slice(2)}`;
</script>

<div class="switch-row">
  <button type="button" role="switch" aria-checked={on ? "true" : "false"} aria-describedby={id} onclick={() => onchange(!on)}>
    <span class="track" aria-hidden="true"><span class="thumb"></span></span>
    <span>Apply this profile when PrismForge connects</span>
    <span class="state" aria-hidden="true">{on ? "On" : "Off"}</span>
  </button>
  <p {id} class="hint">When PrismForge connects, it applies this profile once. PrismForge connects by itself only when “Connect to OpenRGB when PrismForge starts” is on in Settings.</p>
  <p class="hint">Only one profile can do this.</p>
</div>

<style>
  .switch-row {
    display: grid;
    gap: 4px;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    justify-self: start;
    background: transparent;
    border-color: transparent;
    padding: 6px 8px 6px 0;
  }

  .track {
    width: 34px;
    height: 20px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--line-subtle);
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
    transition: transform 120ms ease;
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

  .hint {
    color: var(--text-faint);
    font-size: var(--font-s);
    max-width: 72ch;
  }
</style>
