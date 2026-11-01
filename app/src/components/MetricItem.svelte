<script>
  export let label = "";
  export let value = "—";
  export let unit = "";
  export let meta = "";
  export let delta = null; // { text, dir: 'up' | 'down' | 'flat' }
  export let highlight = false;
</script>

<div class="metric-item" class:highlight>
  <div class="metric-top">
    <span class="metric-label">{label}</span>
    {#if delta && delta.text}
      <span class="metric-delta {delta.dir}">{delta.text}</span>
    {/if}
  </div>
  <div class="metric-main">
    <span class="metric-value num">{value}</span>
    {#if unit}
      <span class="metric-unit">{unit}</span>
    {/if}
  </div>
  {#if meta}
    <div class="metric-meta">{meta}</div>
  {/if}
</div>

<style>
  .metric-item {
    padding: 9px 14px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .metric-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .metric-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metric-delta {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .metric-delta.up {
    color: var(--accent);
  }

  .metric-delta.down {
    color: var(--text-muted);
  }

  .metric-main {
    display: flex;
    align-items: baseline;
    gap: 5px;
    margin-top: 2px;
  }

  .metric-value {
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: var(--lh-tight);
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .metric-item.highlight .metric-value {
    color: var(--accent);
  }

  .metric-unit {
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-muted);
  }

  .metric-meta {
    font-size: var(--text-xs);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
