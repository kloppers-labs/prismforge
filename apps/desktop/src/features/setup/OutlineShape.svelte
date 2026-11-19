<script lang="ts">
  import type { Outline } from "../../lib/api/bindings/Outline";
  import type { Point } from "./geometry";

  const LABEL_GAP = 20;

  let { outline, selected, offset = { x: 0, y: 0 } }: { outline: Outline; selected: boolean; offset?: Point } = $props();

  let at = $derived({ ...outline, x: outline.x + offset.x, y: outline.y + offset.y });
  let w = $derived(at.width / 2);
  let h = $derived(at.height / 2);
</script>

<g class="outline-shape" class:selected data-outline={outline.id} transform="translate({at.x} {at.y}) rotate({at.rotation})">
  {#if outline.kind === "Desk"}
    <rect class="body" x={-w} y={-h} width={at.width} height={Math.min(at.height, 40)} rx="4" />
    <path class="detail" d="M {-w + 30} {-h + Math.min(at.height, 40)} V {h + 40} M {w - 30} {-h + Math.min(at.height, 40)} V {h + 40}" />
  {:else if outline.kind === "Monitor"}
    <rect class="body" x={-w} y={-h} width={at.width} height={at.height} rx="6" />
    <rect class="detail" x={-w + 10} y={-h + 10} width={Math.max(at.width - 20, 1)} height={Math.max(at.height - 20, 1)} rx="3" />
    <path class="detail" d="M 0 {h} V {h + 40} M -60 {h + 40} H 60" />
  {:else}
    <rect class="body" x={-w} y={-h} width={at.width} height={at.height} rx={outline.kind === "Pc" ? 10 : 14} />
    <path class="detail" d="M {-w + 16} {-h + 30} H {w - 16} M {-w + 16} {h - 30} H {w - 16}" />
    {#if outline.kind === "Pc"}
      <circle class="detail" cx="0" cy={-h + 60} r="8" />
    {/if}
  {/if}
  {#if selected}
    <rect class="selection" x={-w - 10} y={-h - 10} width={at.width + 20} height={at.height + 20} rx="8" />
  {/if}
  <text class="name" x="0" y={h + LABEL_GAP + (outline.kind === "Desk" || outline.kind === "Monitor" ? 40 : 0)} text-anchor="middle">{outline.name}</text>
</g>

<style>
  .outline-shape {
    cursor: grab;
  }

  .body,
  .detail {
    fill: none;
    stroke: var(--line-control);
    stroke-width: 1.5;
    stroke-dasharray: 8 6;
  }

  .detail {
    stroke: var(--line-subtle);
    stroke-dasharray: none;
  }

  .name {
    fill: var(--text-faint);
    font-size: 14px;
    font-style: italic;
  }

  .selection {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    stroke-dasharray: 6 4;
  }

  @media (forced-colors: active) {
    .body,
    .detail {
      stroke: CanvasText;
    }

    .name {
      fill: CanvasText;
    }

    .selection {
      stroke: Highlight;
    }
  }
</style>
