<p class="demo" data-demo>
  <span class="badge">Demo mode</span>
  <span class="detail">simulated devices<span class="visually-hidden">. Nothing is sent to hardware.</span></span>
</p>

<style>
  .demo {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    white-space: nowrap;
  }

  .badge {
    padding: 2px var(--space-2);
    border-radius: var(--radius-xs);
    background: var(--demo-stripes);
    color: var(--demo-ink);
    font-weight: var(--weight-heading);
  }

  @media (forced-colors: active) {
    .badge {
      border: 2px dashed CanvasText;
    }
  }

  .detail {
    color: var(--text-dim);
    font-size: var(--font-s);
  }
</style>
