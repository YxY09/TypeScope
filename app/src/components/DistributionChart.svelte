<script>
  import { fmtNum } from "../lib/format.js";
  import TrendChart from "./TrendChart.svelte";

  // Supports two clean modes:
  // 1. mode="proportion": Horizontal stacked composition bar + compact tabular breakdown (ideal for Han/Latin/Digit/Punct composition)
  // 2. mode="histogram": ECharts histogram via chartSpec (ideal for 24h rhythm or 12-bucket interval distribution)
  export let mode = "proportion";
  export let segments = []; // [{ key, label, value, color }]
  export let chartSpec = null;
  export let height = "180px";

  $: total = (segments || []).reduce((acc, s) => acc + (Number(s.value) || 0), 0);
</script>

{#if mode === "histogram"}
  <TrendChart {chartSpec} {height} />
{:else}
  <div class="composition-box">
    {#if total <= 0}
      <div class="comp-empty">所选范围内尚无分类字符数据</div>
    {:else}
      <div class="stacked-bar" role="img" aria-label="输入构成比例">
        {#each segments as seg}
          {@const val = Number(seg.value) || 0}
          {#if val > 0}
            {@const pct = Math.max(1.5, (val * 100) / total)}
            <div
              class="bar-seg"
              style="width: {pct}%; background-color: {seg.color}"
              title="{seg.label}: {fmtNum(val)} ({((val * 100) / total).toFixed(1)}%)"
            ></div>
          {/if}
        {/each}
      </div>

      <div class="comp-legend">
        {#each segments as seg}
          {@const val = Number(seg.value) || 0}
          {@const pct = total > 0 ? ((val * 100) / total).toFixed(1) : "0.0"}
          <div class="legend-row">
            <div class="legend-name">
              <span class="swatch" style="background-color: {seg.color}"></span>
              <span class="label">{seg.label}</span>
            </div>
            <div class="legend-metrics">
              <span class="val num">{fmtNum(val)}</span>
              <span class="pct num">{pct}%</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .composition-box {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .stacked-bar {
    height: 10px;
    width: 100%;
    display: flex;
    gap: 2px;
    border-radius: var(--radius-xs);
    overflow: hidden;
    background: var(--bg-subtle);
  }

  .bar-seg {
    height: 100%;
    transition: width 0.2s ease;
  }

  .comp-legend {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--sp-5);
    row-gap: 6px;
  }

  .legend-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: var(--text-sm);
  }

  .legend-name {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
  }

  .swatch {
    width: 8px;
    height: 8px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .legend-metrics {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
  }

  .val {
    font-weight: 600;
    color: var(--text-primary);
  }

  .pct {
    font-size: var(--text-xs);
    color: var(--text-muted);
    min-width: 40px;
    text-align: right;
  }

  .comp-empty {
    padding: 16px 12px;
    font-size: var(--text-xs);
    color: var(--text-muted);
    background: var(--bg-subtle);
    border-radius: var(--radius-sm);
    text-align: center;
  }
</style>
