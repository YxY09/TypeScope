<script>
  import { onDestroy, onMount } from "svelte";
  import echarts from "../lib/echarts.js";
  import EmptyState from "./EmptyState.svelte";

  export let chartSpec = { empty: true, reason: "暂无数据", option: null };
  export let height = "220px";

  let container = null;
  let instance = null;
  let observer = null;
  let lastOptionHash = "";

  function computeOptionHash(opt) {
    if (!opt) return "";
    try {
      return JSON.stringify({
        s: opt.series?.map((sr) => ({ d: sr.data, c: sr.itemStyle?.color, t: sr.type })),
        x: opt.xAxis?.data
      });
    } catch (_) {
      return String(Date.now());
    }
  }

  function syncChart() {
    if (!container) return;
    if (!chartSpec || chartSpec.empty || !chartSpec.option) {
      if (instance) {
        instance.dispose();
        instance = null;
      }
      lastOptionHash = "";
      return;
    }
    if (!instance) {
      instance = echarts.init(container, null, { renderer: "canvas" });
      lastOptionHash = "";
    }
    const nextHash = computeOptionHash(chartSpec.option);
    if (nextHash !== lastOptionHash) {
      instance.setOption(chartSpec.option, { notMerge: true, lazyUpdate: false });
      lastOptionHash = nextHash;
    }
  }

  $: if (container && chartSpec) {
    syncChart();
  }

  onMount(() => {
    syncChart();
    if (typeof ResizeObserver !== "undefined" && container) {
      observer = new ResizeObserver(() => {
        if (instance) {
          instance.resize();
        }
      });
      observer.observe(container);
    }
  });

  onDestroy(() => {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
    if (instance) {
      instance.dispose();
      instance = null;
    }
    lastOptionHash = "";
  });
</script>

{#if !chartSpec || chartSpec.empty}
  <div class="chart-empty-holder" style="min-height: {height}">
    <EmptyState title={chartSpec?.reason || "暂无趋势数据"} compact={true} />
  </div>
{:else}
  <div class="trend-chart-canvas" bind:this={container} style="height: {height}"></div>
{/if}

<style>
  .trend-chart-canvas {
    width: 100%;
  }

  .chart-empty-holder {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .chart-empty-holder :global(.empty-state) {
    width: 100%;
    height: 100%;
  }
</style>
