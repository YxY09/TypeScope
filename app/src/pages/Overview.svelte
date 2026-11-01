<script>
  import MetricStrip from "../components/MetricStrip.svelte";
  import MetricItem from "../components/MetricItem.svelte";
  import SectionHeader from "../components/SectionHeader.svelte";
  import TrendChart from "../components/TrendChart.svelte";
  import DistributionChart from "../components/DistributionChart.svelte";
  import RankingList from "../components/RankingList.svelte";
  import { fmtNum, fmtDur, fmtMinute, kpm, deltaBadge } from "../lib/format.js";
  import { labelOf, keyZone } from "../lib/keys.js";
  import { buildOverviewTrendOption, buildHourlyRhythmOption } from "../lib/chartAdapter.js";

  export let snap = null;
  export let rangeDays = [];
  export let prevRangeDays = [];
  export let rangeTotals = {};
  export let prevRangeTotals = {};
  export let rangeReport = null;
  export let isSingleDay = true;
  export let theme = "light";
  export let onNavigate = () => {};

  $: view = snap?.view || {};
  $: lifetime = snap?.lifetime || {};
  $: hours = snap?.hours || [];

  // Primary metrics (adapt to selected range; for Today, use realtime snapshot view)
  $: physical = isSingleDay ? (view.physical || 0) : (rangeTotals.physical || 0);
  $: commits = isSingleDay ? (view.commits || 0) : (rangeTotals.commits || 0);
  $: han = isSingleDay ? (view.han || 0) : (rangeTotals.han || 0);
  $: latin = isSingleDay ? (view.latin || 0) : (rangeTotals.latin || 0);
  $: digit = isSingleDay ? (view.digit || 0) : (rangeTotals.digit || 0);
  $: punct = isSingleDay ? (view.punct || 0) : (rangeTotals.punct || 0);
  $: totalChars = han + latin + digit + punct;
  $: activeMs = isSingleDay ? (view.active_ms || 0) : (rangeTotals.active_ms || 0);
  $: backspace = isSingleDay ? (view.backspace || 0) : (rangeTotals.backspace || 0);
  $: sessions = isSingleDay ? (view.sessions || 0) : (rangeTotals.sessions || 0);
  $: avgSpeed = kpm(physical, activeMs);
  $: best10 = isSingleDay ? (view.best_10 || 0) : (rangeReport?.record_10 || rangeTotals.best_10 || 0);
  $: best30 = rangeReport?.record_30 || 0;
  $: best60 = rangeReport?.record_60 || 0;

  // Previous period deltas
  $: prevPhysical = prevRangeTotals.physical || 0;
  $: prevChars = (prevRangeTotals.han || 0) + (prevRangeTotals.latin || 0) + (prevRangeTotals.digit || 0) + (prevRangeTotals.punct || 0);
  $: prevActiveMs = prevRangeTotals.active_ms || 0;
  $: prevSpeed = kpm(prevPhysical, prevActiveMs);

  $: physicalDelta = deltaBadge(physical, prevPhysical);
  $: charDelta = deltaBadge(totalChars, prevChars);
  $: activeDelta = deltaBadge(activeMs, prevActiveMs);
  $: speedDelta = avgSpeed && prevSpeed ? deltaBadge(avgSpeed, prevSpeed) : { text: "", dir: "flat" };

  // Main Volume Trend Chart
  $: mainTrendSpec = buildOverviewTrendOption({
    isSingleDay,
    hours,
    days: rangeDays,
    prevDays: prevRangeDays,
    theme
  });

  // 24h Rhythm Chart (if multi-day, aggregate hour_grid into 24 hours; if single day, use snap.hours)
  $: rhythmHours = (() => {
    if (isSingleDay || !rangeReport?.hour_grid) return hours;
    const agg = Array.from({ length: 24 }, () => ({ physical: 0, active_ms: 0 }));
    for (const row of rangeReport.hour_grid) {
      if (Array.isArray(row)) {
        for (let h = 0; h < 24; h++) {
          agg[h].physical += Number(row[h] || 0);
        }
      }
    }
    return agg;
  })();

  $: hourlyRhythmSpec = buildHourlyRhythmOption(rhythmHours, theme);

  $: peakHourInfo = (() => {
    let bestH = -1;
    let bestVal = 0;
    for (let h = 0; h < 24; h++) {
      const v = rhythmHours[h]?.physical || 0;
      if (v > bestVal) {
        bestVal = v;
        bestH = h;
      }
    }
    if (bestH < 0 || bestVal === 0) return "暂无峰值时段";
    return `峰值 ${String(bestH).padStart(2, "0")}:00 (${fmtNum(bestVal)})`;
  })();

  // Input Composition Segments
  $: compositionSegments = [
    { key: "han", label: "汉字", value: han, color: "var(--cat-han)" },
    { key: "latin", label: "英文", value: latin, color: "var(--cat-latin)" },
    { key: "digit", label: "数字", value: digit, color: "var(--cat-digit)" },
    { key: "punct", label: "标点", value: punct, color: "var(--cat-punct)" }
  ];

  // Top Keys Ranking (always sort descending by count)
  $: rawKeys = [...(isSingleDay ? (snap?.keys || []) : (rangeReport?.top_keys || []))].sort(
    (a, b) => (Number(b.n) || 0) - (Number(a.n) || 0)
  );
  $: topKeyItems = rawKeys.slice(0, 12).map((k) => ({
    key: k.vk,
    label: labelOf(k.vk),
    badge: keyZone(k.vk),
    value: k.n
  }));

  $: correctionRate = physical > 0 ? `${((backspace * 100) / physical).toFixed(1)}%` : "0.0%";
</script>

<div class="page-container">
  <!-- 第一部分：统一核心指标带（无独立卡片） -->
  <MetricStrip columns={5}>
    <MetricItem
      label="物理按键"
      value={fmtNum(physical)}
      unit="次"
      delta={physicalDelta}
      meta="退格 {fmtNum(backspace)} 次 ({correctionRate})"
      highlight={true}
    />
    <MetricItem
      label="提交字符"
      value={fmtNum(totalChars)}
      unit="字"
      delta={charDelta}
      meta="上屏 {fmtNum(commits)} 次 · 汉字 {fmtNum(han)}"
    />
    <MetricItem
      label="活跃时间"
      value={fmtDur(activeMs)}
      delta={activeDelta}
      meta="连续输入 {fmtNum(sessions)} 段"
    />
    <MetricItem
      label="平均速度"
      value={avgSpeed !== null ? fmtNum(avgSpeed) : "—"}
      unit={avgSpeed !== null ? "KPM" : ""}
      delta={speedDelta}
      meta="有效活跃窗口折算"
    />
    <MetricItem
      label="短时速度纪录"
      value={best10 > 0 ? fmtNum(best10) : "—"}
      unit={best10 > 0 ? "KPM" : ""}
      meta="10s 峰值 · 30s {best30 || '—'} · 60s {best60 || '—'}"
    />
  </MetricStrip>

  <!-- 第二部分：主要数据分析（左：输入量趋势；右：输入构成 + 每小时输入节奏） -->
  <div class="split-grid">
    <section class="section-block">
      <SectionHeader
        title={isSingleDay ? "今日分时输入趋势" : "输入量历史趋势"}
        subtitle={isSingleDay ? "按小时统计物理按键与中英字符提交" : "物理按键趋势及上期同跨度基准对比"}
        meta={isSingleDay ? "24 小时序列" : `共 ${rangeDays.length} 天`}
      />
      <div class="chart-stage">
        <TrendChart chartSpec={mainTrendSpec} height="206px" />
      </div>
    </section>

    <div class="right-stack">
      <section class="section-block">
        <SectionHeader
          title="输入构成"
          subtitle="汉字、英文、数字与标点比例"
          meta="共 {fmtNum(totalChars)} 字"
        >
          <button type="button" class="link-btn" on:click={() => onNavigate("text")}>
            文字详情 →
          </button>
        </SectionHeader>
        <DistributionChart mode="proportion" segments={compositionSegments} />
      </section>

      <section class="section-block hairline-top">
        <SectionHeader
          title="每小时输入节奏"
          subtitle="00:00 — 23:00 活跃分布"
          meta={peakHourInfo}
        />
        <DistributionChart mode="histogram" chartSpec={hourlyRhythmSpec} height="92px" />
      </section>
    </div>
  </div>

  <!-- 第三部分：辅助统计（左：紧凑常用按键排行；右：克制的生涯与会话统计） -->
  <div class="split-grid hairline-top">
    <section class="section-block">
      <SectionHeader
        title="常用按键排行"
        subtitle="按物理按下次数排序 (Top 12)"
        meta="总计 {fmtNum(physical)} 次"
      >
        <button type="button" class="link-btn" on:click={() => onNavigate("analytics", "keyboard")}>
          完整键盘热力 →
        </button>
      </SectionHeader>
      <RankingList
        items={topKeyItems}
        total={physical}
        columns={2}
        maxItems={12}
        emptyText="所选范围内暂无按键排行数据"
      />
    </section>

    <section class="section-block">
      <SectionHeader
        title="生涯与活动摘要"
        subtitle="本地历史累计与今日活动窗口"
        meta="连续打卡 {snap?.streak || 0} 天"
      />
      <div class="lifetime-grid">
        <div class="kv-row">
          <span class="kv-label">生涯累计物理按键</span>
          <span class="kv-val num">{fmtNum(lifetime.physical || 0)} 次</span>
        </div>
        <div class="kv-row">
          <span class="kv-label">生涯累计中英字符</span>
          <span class="kv-val num">{fmtNum((lifetime.han || 0) + (lifetime.latin || 0))} 字</span>
        </div>
        <div class="kv-row">
          <span class="kv-label">生涯累计上屏提交</span>
          <span class="kv-val num">{fmtNum(lifetime.commits || 0)} 次</span>
        </div>
        <div class="kv-row">
          <span class="kv-label">生涯累计活跃时长</span>
          <span class="kv-val num">{fmtDur(lifetime.active_ms || 0)}</span>
        </div>
        <div class="kv-row">
          <span class="kv-label">今日起止活跃时刻</span>
          <span class="kv-val num">{fmtMinute(view.first_minute)} — {fmtMinute(view.last_minute)}</span>
        </div>
        <div class="kv-row">
          <span class="kv-label">单次最长连续输入</span>
          <span class="kv-val num">{fmtDur(isSingleDay ? (view.longest_ms || 0) : (rangeReport?.record_longest_ms || 0))}</span>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .chart-stage {
    padding-top: 4px;
  }

  .right-stack {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .link-btn {
    font-size: var(--text-xs);
    color: var(--accent);
    font-weight: 500;
  }

  .link-btn:hover {
    text-decoration: underline;
  }

  .lifetime-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--sp-5);
    row-gap: 4px;
  }

  .kv-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    border-bottom: 1px solid var(--border-subtle);
    font-size: var(--text-sm);
  }

  .kv-label {
    color: var(--text-secondary);
  }

  .kv-val {
    font-weight: 600;
    color: var(--text-primary);
  }
</style>
