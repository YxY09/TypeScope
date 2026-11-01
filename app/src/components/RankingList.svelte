<script>
  import { fmtNum } from "../lib/format.js";

  export let items = []; // [{ key, label, value, sub, badge }]
  export let total = 0;
  export let columns = 1; // 1 or 2
  export let maxItems = 12;
  export let selectedKey = null;
  export let onSelect = null;
  export let emptyText = "暂无排行数据";
  export let barColor = "var(--accent)";

  $: visibleItems = (items || []).slice(0, maxItems);
  $: maxVal = visibleItems.reduce((m, it) => Math.max(m, Number(it.value) || 0), 0) || 1;
  $: sumVal = total > 0 ? total : visibleItems.reduce((s, it) => s + (Number(it.value) || 0), 0) || 1;
</script>

{#if visibleItems.length === 0}
  <div class="ranking-empty">{emptyText}</div>
{:else}
  <div class="ranking-list" class:two-col={columns === 2}>
    {#each visibleItems as item, idx}
      {@const pctOfMax = Math.max(3, Math.round(((Number(item.value) || 0) * 100) / maxVal))}
      {@const sharePct = (((Number(item.value) || 0) * 100) / sumVal).toFixed(1)}
      <button
        type="button"
        class="ranking-row"
        class:interactive={Boolean(onSelect)}
        class:selected={selectedKey !== null && selectedKey === item.key}
        on:click={() => onSelect && onSelect(item)}
      >
        <span class="rank-idx mono">{idx + 1}</span>
        <span class="rank-label-wrap">
          <span class="rank-label" title={item.label}>{item.label}</span>
          {#if item.badge}
            <span class="rank-badge">{item.badge}</span>
          {/if}
        </span>
        <span class="rank-bar-track">
          <span class="rank-bar-fill" style="width: {pctOfMax}%; background: {barColor}"></span>
        </span>
        <span class="rank-val num">{fmtNum(item.value)}</span>
        <span class="rank-share num">{item.sub ? item.sub : `${sharePct}%`}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .ranking-list {
    display: grid;
    grid-template-columns: 1fr;
    gap: 2px;
  }

  .ranking-list.two-col {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--sp-5);
    row-gap: 2px;
  }

  .ranking-row {
    display: grid;
    grid-template-columns: 18px minmax(84px, auto) 1fr auto 44px;
    align-items: center;
    gap: 8px;
    height: 27px;
    padding: 0 6px;
    border-radius: var(--radius-xs);
    text-align: left;
    cursor: default;
    border-bottom: 1px solid var(--border-subtle);
    min-width: 0;
  }

  .ranking-row.interactive {
    cursor: pointer;
  }

  .ranking-row.interactive:hover {
    background: var(--bg-hover);
  }

  .ranking-row.selected {
    background: var(--accent-soft);
  }

  .rank-idx {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .rank-label-wrap {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
    overflow: hidden;
  }

  .rank-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .rank-badge {
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-subtle);
    padding: 1px 4px;
    border-radius: 2px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .rank-bar-track {
    height: 4px;
    width: 100%;
    min-width: 32px;
    background: var(--bg-subtle);
    border-radius: 2px;
    overflow: hidden;
    position: relative;
  }

  .rank-bar-fill {
    display: block;
    height: 100%;
    border-radius: 2px;
  }

  .rank-val {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
    text-align: right;
    white-space: nowrap;
  }

  .rank-share {
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-align: right;
    white-space: nowrap;
  }

  .ranking-empty {
    padding: 16px 12px;
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-align: center;
    background: var(--bg-subtle);
    border-radius: var(--radius-sm);
  }

  @media (max-width: 900px) {
    .ranking-list.two-col {
      grid-template-columns: 1fr;
    }
  }
</style>
