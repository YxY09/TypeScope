<script>
  import MetricStrip from "../components/MetricStrip.svelte";
  import MetricItem from "../components/MetricItem.svelte";
  import SectionHeader from "../components/SectionHeader.svelte";
  import TrendChart from "../components/TrendChart.svelte";
  import DistributionChart from "../components/DistributionChart.svelte";
  import Heatmap from "../components/Heatmap.svelte";
  import RankingList from "../components/RankingList.svelte";
  import DataTable from "../components/DataTable.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import {
    fmtNum,
    fmtDur,
    fmtDay,
    fmtMinute,
    kpm,
    WEEKDAY_NAMES,
    weekdayMon,
    INTERVAL_LABELS
  } from "../lib/format.js";
  import { labelOf, handOf, keyZone, shortcutLabel } from "../lib/keys.js";
  import {
    buildSpeedTrendOption,
    buildIntervalDistributionOption,
    buildItemTrendOption
  } from "../lib/chartAdapter.js";

  export let subTab = "speed"; // 'speed' | 'keyboard' | 'habits' | 'apps'
  export let snap = null;
  export let rangeDays = [];
  export let rangeTotals = {};
  export let rangeReport = null;
  export let currentRange = { start: 0, end: 0 };
  export let isSingleDay = true;
  export let theme = "light";
  export let selectedVk = 32;
  export let keyTrendPoints = [];

  export let onChangeSubTab = () => {};
  export let onSelectKey = () => {};
  export let onEnableAppStats = () => {};
  export let onOpenExclude = () => {};

  const subNavItems = [
    { id: "speed", label: "速度与节奏" },
    { id: "keyboard", label: "键盘热力" },
    { id: "habits", label: "习惯与日历" },
    { id: "apps", label: "程序分类" }
  ];

  $: view = snap?.view || {};
  $: hours = snap?.hours || [];
  $: physical = isSingleDay ? (view.physical || 0) : (rangeTotals.physical || 0);
  $: activeMs = isSingleDay ? (view.active_ms || 0) : (rangeTotals.active_ms || 0);
  $: avgSpeed = kpm(physical, activeMs);
  $: best10 = isSingleDay ? (view.best_10 || 0) : (rangeReport?.record_10 || rangeTotals.best_10 || 0);
  $: best30 = rangeReport?.record_30 || 0;
  $: best60 = rangeReport?.record_60 || 0;
  $: sessions = isSingleDay ? (view.sessions || 0) : (rangeTotals.sessions || 0);
  $: longestMs = isSingleDay ? (view.longest_ms || 0) : (rangeReport?.record_longest_ms || 0);

  // 1. Speed & Rhythm Data
  $: speedTrendSpec = buildSpeedTrendOption({
    isSingleDay,
    hours,
    days: rangeDays,
    theme
  });

  $: intervals = isSingleDay ? (snap?.intervals || []) : (rangeReport?.intervals || []);
  $: intervalSpec = buildIntervalDistributionOption(intervals, theme);

  $: fastRatio = (() => {
    const total = (intervals || []).reduce((a, b) => a + Number(b || 0), 0);
    if (!total) return "0.0%";
    const under120 = (Number(intervals[0] || 0) + Number(intervals[1] || 0) + Number(intervals[2] || 0) + Number(intervals[3] || 0));
    return `${((under120 * 100) / total).toFixed(1)}%`;
  })();

  $: speedTableColumns = [
    { key: "date", label: "日期" },
    { key: "physical", label: "物理按键", align: "right" },
    { key: "active", label: "活跃时长", align: "right" },
    { key: "avgKpm", label: "日均速度", align: "right" },
    { key: "best10", label: "10s 峰值", align: "right" },
    { key: "best30", label: "30s 峰值", align: "right" },
    { key: "best60", label: "60s 峰值", align: "right" },
    { key: "sessions", label: "输入段数", align: "right" },
    { key: "window", label: "起止窗口", align: "right" }
  ];

  $: speedTableRows = [...(rangeDays || [])]
    .reverse()
    .slice(0, 14)
    .map((d) => {
      const spd = kpm(d.physical, d.active_ms);
      return {
        id: d.day,
        date: fmtDay(d.day),
        physical: fmtNum(d.physical),
        active: fmtDur(d.active_ms),
        avgKpm: spd ? `${spd} KPM` : "—",
        best10: d.best_10 ? `${d.best_10} KPM` : "—",
        best30: d.best_30 ? `${d.best_30} KPM` : "—",
        best60: d.best_60 ? `${d.best_60} KPM` : "—",
        sessions: fmtNum(d.sessions || 0),
        window: `${fmtMinute(d.first_minute)} — ${fmtMinute(d.last_minute)}`
      };
    });

  // 2. Keyboard Heatmap Data
  $: rawKeys = [...(isSingleDay ? (snap?.keys || []) : (rangeReport?.top_keys || []))].sort(
    (a, b) => (Number(b.n) || 0) - (Number(a.n) || 0)
  );
  $: keyMap = new Map((rawKeys || []).map((k) => [Number(k.vk), Number(k.n) || 0]));
  $: activeKeyCount = rawKeys.filter((k) => Number(k.n) > 0).length;
  $: topKeyObj = rawKeys[0] || null;

  $: handStats = (() => {
    let left = 0;
    let right = 0;
    for (const k of rawKeys) {
      const h = handOf(Number(k.vk));
      if (h === "left") left += Number(k.n) || 0;
      else if (h === "right") right += Number(k.n) || 0;
    }
    const sum = left + right || 1;
    return {
      left,
      right,
      leftPct: Math.round((left * 100) / sum),
      rightPct: Math.round((right * 100) / sum)
    };
  })();

  $: backspaceCount = isSingleDay ? (view.backspace || 0) : (rangeTotals.backspace || 0);
  $: deleteCount = rangeReport?.delete_total || 0;
  $: spaceCount = rangeReport?.space_total || keyMap.get(32) || 0;
  $: enterCount = rangeReport?.enter_total || keyMap.get(13) || 0;

  $: selectedKeyCount = keyMap.get(Number(selectedVk)) || 0;
  $: selectedKeyShare = physical > 0 ? `${((selectedKeyCount * 100) / physical).toFixed(2)}%` : "0.00%";
  $: selectedKeyLabel = labelOf(Number(selectedVk));
  $: selectedKeyZone = keyZone(Number(selectedVk));
  $: selectedKeyHand = handOf(Number(selectedVk)) === "left" ? "左手区" : handOf(Number(selectedVk)) === "right" ? "右手区" : "中区";

  $: singleKeyTrendSpec = buildItemTrendOption({
    points: keyTrendPoints,
    start: currentRange.start,
    end: currentRange.end,
    label: selectedKeyLabel,
    theme
  });

  $: rawShortcuts = isSingleDay ? (snap?.shortcuts || []) : (rangeReport?.top_shortcuts || []);
  $: shortcutItems = rawShortcuts.slice(0, 10).map((s, i) => ({
    key: `${s.mods}-${s.vk}-${i}`,
    label: shortcutLabel(s.mods, s.vk),
    value: s.n
  }));

  $: keyRankingItems = rawKeys.slice(0, 14).map((k) => ({
    key: k.vk,
    label: labelOf(k.vk),
    badge: keyZone(k.vk),
    value: k.n
  }));

  // 3. Habits & Calendar Data
  $: calendarDays = rangeReport?.calendar_days || snap?.days || [];
  $: activeCalendarDaysCount = calendarDays.filter((d) => Number(d.physical) > 0).length;
  $: bestCalendarDay = calendarDays.reduce(
    (best, d) => (Number(d.physical) > best.physical ? { day: d.day, physical: Number(d.physical) } : best),
    { day: 0, physical: 0 }
  );

  $: weekdayRanking = (() => {
    const totals = [0, 0, 0, 0, 0, 0, 0];
    const grid = rangeReport?.hour_grid;
    if (Array.isArray(grid) && grid.length === 7) {
      for (let w = 0; w < 7; w++) {
        totals[w] = (grid[w] || []).reduce((a, b) => a + Number(b || 0), 0);
      }
    } else {
      for (const d of rangeDays) {
        totals[weekdayMon(d.day)] += Number(d.physical) || 0;
      }
    }
    return WEEKDAY_NAMES.map((name, idx) => ({
      key: idx,
      label: name,
      value: totals[idx]
    }));
  })();

  // 4. Applications Data
  $: appStatsOn = Boolean(snap?.flags?.app_stats);
  $: rawApps = isSingleDay ? (snap?.apps || []) : (rangeReport?.top_apps || []);
  $: appRankingItems = rawApps.map((a) => ({
    key: a.exe,
    label: a.exe,
    value: a.physical,
    sub: fmtDur(a.active_ms)
  }));

  $: appTableColumns = [
    { key: "rank", label: "#", width: "36px" },
    { key: "exe", label: "应用程序 (进程名)" },
    { key: "physical", label: "物理按键", align: "right" },
    { key: "share", label: "按键占比", align: "right" },
    { key: "active", label: "活跃时长", align: "right" },
    { key: "speed", label: "平均速度", align: "right" }
  ];

  $: totalAppPhysical = rawApps.reduce((s, a) => s + (Number(a.physical) || 0), 0) || 1;
  $: appTableRows = rawApps.map((a, idx) => {
    const spd = kpm(a.physical, a.active_ms);
    return {
      id: a.exe,
      rank: idx + 1,
      exe: a.exe,
      physical: fmtNum(a.physical),
      share: `${(((Number(a.physical) || 0) * 100) / totalAppPhysical).toFixed(1)}%`,
      active: fmtDur(a.active_ms),
      speed: spd ? `${spd} KPM` : "—"
    };
  });
</script>

<div class="page-container">
  <!-- 二级分析维度切换栏 -->
  <div class="subnav-bar">
    <div class="subnav-tabs" role="tablist" aria-label="分析维度">
      {#each subNavItems as item}
        <button
          type="button"
          role="tab"
          class="subnav-btn"
          class:active={subTab === item.id}
          on:click={() => onChangeSubTab(item.id)}
        >
          {item.label}
        </button>
      {/each}
    </div>
    <div class="subnav-hint">
      {#if subTab === "speed"}
        速度分母固定为完整窗口时长，避免短促连击产生虚高纪录
      {:else if subTab === "keyboard"}
        基于底层 Raw Input 物理扫描码统计，区分左右修饰键
      {:else if subTab === "habits"}
        展示近 26 周连续活跃度与 7×24 小时时段分布
      {:else}
        仅在本地统计前台进程可执行文件名，不采集窗口标题
      {/if}
    </div>
  </div>

  {#if subTab === "speed"}
    <!-- 维度 1：速度与节奏 -->
    <MetricStrip columns={5}>
      <MetricItem
        label="平均打字速度"
        value={avgSpeed !== null ? fmtNum(avgSpeed) : "—"}
        unit={avgSpeed !== null ? "KPM" : ""}
        meta="活跃时长 {fmtDur(activeMs)}"
        highlight={true}
      />
      <MetricItem
        label="10 秒爆发纪录"
        value={best10 > 0 ? fmtNum(best10) : "—"}
        unit={best10 > 0 ? "KPM" : ""}
        meta="需窗口内 ≥20 次击键"
      />
      <MetricItem
        label="30s / 60s 持续纪录"
        value="{best30 || '—'} / {best60 || '—'}"
        unit="KPM"
        meta="中长时稳定输出能力"
      />
      <MetricItem
        label="高速连击占比 (<120ms)"
        value={fastRatio}
        meta="反映肌肉记忆连贯度"
      />
      <MetricItem
        label="最长单次连续输入"
        value={fmtDur(longestMs)}
        meta="共 {fmtNum(sessions)} 段连续会话"
      />
    </MetricStrip>

    <div class="split-grid">
      <section class="section-block">
        <SectionHeader
          title={isSingleDay ? "今日每小时打字速度" : "平均速度与 10 秒峰值趋势"}
          subtitle={isSingleDay ? "各活跃小时内的有效平均速度 (KPM)" : "按日统计有效均速及短时爆发上限"}
        />
        <TrendChart chartSpec={speedTrendSpec} height="230px" />
      </section>

      <section class="section-block">
        <SectionHeader
          title="按键间隔分布 (Inter-keystroke Intervals)"
          subtitle="相邻两次物理击键的时间差分布 (12 桶)"
          meta="<120ms 占 {fastRatio}"
        />
        <DistributionChart mode="histogram" chartSpec={intervalSpec} height="230px" />
      </section>
    </div>

    <section class="section-block hairline-top">
      <SectionHeader
        title="每日速度与会话明细"
        subtitle="所选时间范围内的速度纪录、输入段数及起止活跃窗口"
        meta="最近 {speedTableRows.length} 天"
      />
      <DataTable columns={speedTableColumns} rows={speedTableRows} />
    </section>

  {:else if subTab === "keyboard"}
    <!-- 维度 2：键盘热力 -->
    <MetricStrip columns={5}>
      <MetricItem
        label="最高频按键"
        value={topKeyObj ? labelOf(topKeyObj.vk) : "—"}
        meta={topKeyObj ? `${fmtNum(topKeyObj.n)} 次 (${physical > 0 ? ((topKeyObj.n * 100) / physical).toFixed(1) : 0}%)` : "尚无记录"}
        highlight={true}
      />
      <MetricItem
        label="左右手负载比"
        value="{handStats.leftPct}% : {handStats.rightPct}%"
        meta="左 {fmtNum(handStats.left)} · 右 {fmtNum(handStats.right)}"
      />
      <MetricItem
        label="空格与回车"
        value="{fmtNum(spaceCount)} / {fmtNum(enterCount)}"
        meta="Space 与 Enter 触发次数"
      />
      <MetricItem
        label="退格与删除"
        value="{fmtNum(backspaceCount)} / {fmtNum(deleteCount)}"
        meta="Bksp 与 Del 修正统计"
      />
      <MetricItem
        label="触发键位覆盖"
        value={fmtNum(activeKeyCount)}
        unit="/ 104 键"
        meta="所选周期内有记录的键位"
      />
    </MetricStrip>

    <section class="section-block">
      <SectionHeader
        title="104 键位频率热力分布"
        subtitle="采用紧凑平面键位矩阵呈现，点击任意键位可检视单键历史趋势"
        meta="当前选中: {selectedKeyLabel}"
      />
      <Heatmap
        mode="keyboard"
        keyCounts={rawKeys}
        {selectedVk}
        onSelectKey={(vk) => onSelectKey(vk)}
      />
    </section>

    <div class="split-grid hairline-top">
      <section class="section-block">
        <SectionHeader
          title="单键深度检视：{selectedKeyLabel}"
          subtitle="{selectedKeyZone} · {selectedKeyHand} · VK_{selectedVk}"
          meta="{fmtNum(selectedKeyCount)} 次 ({selectedKeyShare})"
        />
        <TrendChart chartSpec={singleKeyTrendSpec} height="190px" />
      </section>

      <div class="split-equal">
        <section class="section-block">
          <SectionHeader
            title="高频键位 Top 14"
            subtitle="点击行可切换单键趋势"
          />
          <RankingList
            items={keyRankingItems}
            total={physical}
            maxItems={10}
            selectedKey={selectedVk}
            onSelect={(item) => onSelectKey(item.key)}
          />
        </section>

        <section class="section-block">
          <SectionHeader
            title="常用组合快捷键"
            subtitle="修饰键 + 普通键组合"
          />
          <RankingList
            items={shortcutItems}
            maxItems={10}
            emptyText="暂无快捷键触发记录"
          />
        </section>
      </div>
    </div>

  {:else if subTab === "habits"}
    <!-- 维度 3：习惯与日历 -->
    <MetricStrip columns={5}>
      <MetricItem
        label="当前连续打卡"
        value={fmtNum(snap?.streak || 0)}
        unit="天"
        meta="每日有物理击键即计入"
        highlight={true}
      />
      <MetricItem
        label="近半年活跃天数"
        value={fmtNum(activeCalendarDaysCount)}
        unit="天"
        meta="过去 26 周内有效记录天"
      />
      <MetricItem
        label="单日最高按键"
        value={bestCalendarDay.physical > 0 ? fmtNum(bestCalendarDay.physical) : "—"}
        unit={bestCalendarDay.physical > 0 ? "次" : ""}
        meta={bestCalendarDay.day ? fmtDay(bestCalendarDay.day) : "尚无峰值"}
      />
      <MetricItem
        label="今日首次击键"
        value={fmtMinute(view.first_minute)}
        meta="今日最早活跃时刻"
      />
      <MetricItem
        label="今日末次击键"
        value={fmtMinute(view.last_minute)}
        meta="今日最近活跃时刻"
      />
    </MetricStrip>

    <section class="section-block">
      <SectionHeader
        title="26 周打字活跃日历"
        subtitle="按自然周纵向排列，直观呈现长期输入连续性与强度"
        meta="共 {activeCalendarDaysCount} 个活跃日"
      />
      <div class="panel-pad">
        <Heatmap mode="calendar" {calendarDays} today={snap?.day || currentRange.end} />
      </div>
    </section>

    <div class="split-grid hairline-top">
      <section class="section-block">
        <SectionHeader
          title="星期 × 小时活跃密度矩阵 (7 × 24)"
          subtitle="所选时间范围内各星期与小时交叉的击键强度分布"
        />
        <div class="panel-pad">
          <Heatmap mode="hour-grid" hourGrid={rangeReport?.hour_grid || []} />
        </div>
      </section>

      <section class="section-block">
        <SectionHeader
          title="星期输入规律"
          subtitle="周一至周日累计按键分布"
        />
        <RankingList
          items={weekdayRanking}
          maxItems={7}
          emptyText="暂无星期分布数据"
        />
      </section>
    </div>

  {:else if subTab === "apps"}
    <!-- 维度 4：程序分类 -->
    {#if !appStatsOn && rawApps.length === 0}
      <EmptyState
        title="程序分类统计当前未开启"
        description="开启后，TypeScope 会在本地按前台进程名（如 Code.exe、WeChat.exe）聚合您的按键次数与活跃时间，绝不读取窗口标题或文档内容。"
      >
        <div style="margin-top: 10px; display: flex; gap: 8px;">
          <button type="button" class="btn btn-primary" on:click={onEnableAppStats}>
            开启程序分类统计
          </button>
          <button type="button" class="btn" on:click={onOpenExclude}>
            配置排除进程名单
          </button>
        </div>
      </EmptyState>
    {:else}
      <div class="split-grid">
        <section class="section-block">
          <SectionHeader
            title="应用程序输入明细"
            subtitle="按前台进程统计物理按键、占比、活跃时长与平均打字速度"
            meta="共 {rawApps.length} 个应用"
          >
            <button type="button" class="btn" on:click={onOpenExclude}>
              编辑排除名单
            </button>
          </SectionHeader>
          <DataTable
            columns={appTableColumns}
            rows={appTableRows}
            emptyText="所选时间范围内尚无应用分类记录"
          />
        </section>

        <section class="section-block">
          <SectionHeader
            title="应用输入量占比排行"
            subtitle="Top 12 进程按键负荷比例"
          />
          <RankingList
            items={appRankingItems}
            total={totalAppPhysical}
            maxItems={12}
            emptyText="尚无应用排行数据"
          />
        </section>
      </div>
    {/if}
  {/if}
</div>

<style>
  .subnav-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .subnav-tabs {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .subnav-btn {
    height: 28px;
    padding: 0 12px;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    transition: all 0.12s ease;
  }

  .subnav-btn:hover:not(.active) {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .subnav-btn.active {
    color: var(--accent-fg);
    background: var(--accent);
    border-color: var(--accent);
    font-weight: 600;
  }

  .subnav-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .panel-pad {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 12px 14px;
  }

  @media (max-width: 900px) {
    .subnav-hint {
      display: none;
    }
  }
</style>
