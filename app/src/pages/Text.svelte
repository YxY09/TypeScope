<script>
  import MetricStrip from "../components/MetricStrip.svelte";
  import MetricItem from "../components/MetricItem.svelte";
  import SectionHeader from "../components/SectionHeader.svelte";
  import DistributionChart from "../components/DistributionChart.svelte";
  import RankingList from "../components/RankingList.svelte";
  import TrendChart from "../components/TrendChart.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import { fmtNum } from "../lib/format.js";
  import { buildItemTrendOption } from "../lib/chartAdapter.js";

  export let snap = null;
  export let rangeTotals = {};
  export let rangeReport = null;
  export let currentRange = { start: 0, end: 0 };
  export let isSingleDay = true;
  export let theme = "light";

  // On-demand trend query state
  export let activeQuery = "";
  export let activeQueryTable = "char_freq"; // 'char_freq' | 'word_freq'
  export let textTrendPoints = [];
  export let onQueryTextTrend = () => {};
  export let onClearTextQuery = () => {};
  export let onToggleFlag = () => {};

  let searchInput = "";
  let searchTable = "char_freq";

  const PUNCT_REGEX = /^[\u3000-\u303F\uFF00-\uFFEF\u2000-\u206F\p{P}\p{S}]+$/u;

  $: view = snap?.view || {};
  $: textStatsOn = Boolean(snap?.flags?.text_stats);
  $: segmentOn = Boolean(snap?.flags?.segment);

  $: han = isSingleDay ? (view.han || 0) : (rangeTotals.han || 0);
  $: latin = isSingleDay ? (view.latin || 0) : (rangeTotals.latin || 0);
  $: digit = isSingleDay ? (view.digit || 0) : (rangeTotals.digit || 0);
  $: punct = isSingleDay ? (view.punct || 0) : (rangeTotals.punct || 0);
  $: commits = isSingleDay ? (view.commits || 0) : (rangeTotals.commits || 0);
  $: physical = isSingleDay ? (view.physical || 0) : (rangeTotals.physical || 0);
  $: totalChars = han + latin + digit + punct;

  $: rawChars = isSingleDay ? (snap?.chars || []) : (rangeReport?.top_chars || []);
  $: rawWords = isSingleDay ? (snap?.words || []) : (rangeReport?.top_words || []);

  // Separate Han characters from Punctuation marks cleanly
  $: hanCharItems = (rawChars || [])
    .filter((c) => !PUNCT_REGEX.test(c.k))
    .map((c) => ({
      key: c.k,
      label: c.k,
      value: c.n
    }));

  $: punctCharItems = (rawChars || [])
    .filter((c) => PUNCT_REGEX.test(c.k))
    .map((c) => ({
      key: c.k,
      label: c.k,
      badge: "标点",
      value: c.n
    }));

  $: wordItems = (rawWords || []).map((w) => ({
    key: w.k,
    label: w.k,
    badge: `${Array.from(w.k).length}字`,
    value: w.n
  }));

  $: compositionSegments = [
    { key: "han", label: "汉字", value: han, color: "var(--cat-han)" },
    { key: "latin", label: "英文", value: latin, color: "var(--cat-latin)" },
    { key: "digit", label: "数字", value: digit, color: "var(--cat-digit)" },
    { key: "punct", label: "标点符号", value: punct, color: "var(--cat-punct)" }
  ];

  $: keyPerChar = totalChars > 0 ? (physical / totalChars).toFixed(2) : "—";
  $: charPerCommit = commits > 0 ? (totalChars / commits).toFixed(1) : "—";

  function triggerSearch() {
    const q = searchInput.trim();
    if (!q) return;
    onQueryTextTrend(searchTable, q);
  }

  function selectFromList(table, item) {
    searchTable = table;
    searchInput = item.key;
    onQueryTextTrend(table, item.key);
  }

  $: queryTrendSpec = activeQuery
    ? buildItemTrendOption({
        points: textTrendPoints,
        start: currentRange.start,
        end: currentRange.end,
        label: activeQuery,
        theme
      })
    : null;
</script>

<div class="page-container">
  <!-- 第一部分：字符类别核心指标带 -->
  <MetricStrip columns={5}>
    <MetricItem
      label="提交总字符"
      value={fmtNum(totalChars)}
      unit="字"
      meta="上屏提交 {fmtNum(commits)} 次 (均 {charPerCommit} 字/次)"
      highlight={true}
    />
    <MetricItem
      label="汉字输入量"
      value={fmtNum(han)}
      unit="字"
      meta="占总字符 {totalChars > 0 ? ((han * 100) / totalChars).toFixed(1) : '0.0'}%"
    />
    <MetricItem
      label="英文输入量"
      value={fmtNum(latin)}
      unit="字符"
      meta="占总字符 {totalChars > 0 ? ((latin * 100) / totalChars).toFixed(1) : '0.0'}%"
    />
    <MetricItem
      label="数字与标点"
      value="{fmtNum(digit)} / {fmtNum(punct)}"
      meta="数字 {fmtNum(digit)} · 标点 {fmtNum(punct)}"
    />
    <MetricItem
      label="综合码长 (击键/字符)"
      value={keyPerChar}
      unit={keyPerChar !== "—" ? "键/字" : ""}
      meta="物理按键与上屏字符比"
    />
  </MetricStrip>

  <!-- 采集状态与按需趋势检索工具条 -->
  <div class="toolbar-strip">
    <div class="toolbar-left">
      <span class="status-tag" class:on={textStatsOn}>
        字频统计：{textStatsOn ? "已开启" : "未开启"}
      </span>
      <span class="status-tag" class:on={segmentOn}>
        n-gram 词组：{segmentOn ? "已开启" : "未开启"}
      </span>
      {#if !textStatsOn}
        <button type="button" class="btn btn-primary" on:click={() => onToggleFlag("text_stats", true)}>
          开启字频统计
        </button>
      {:else if !segmentOn}
        <button type="button" class="btn" on:click={() => onToggleFlag("segment", true)}>
          开启词组切分
        </button>
      {/if}
    </div>

    <form class="query-form" on:submit|preventDefault={triggerSearch}>
      <select class="query-select" bind:value={searchTable}>
        <option value="char_freq">单字 / 标点</option>
        <option value="word_freq">词组 (n-gram)</option>
      </select>
      <input
        type="text"
        class="query-input"
        placeholder="输入单字或词组查询历史趋势（或直接点击下方排行项）"
        bind:value={searchInput}
      />
      <button type="submit" class="btn">查询趋势</button>
      {#if activeQuery}
        <button
          type="button"
          class="btn"
          on:click={() => {
            searchInput = "";
            onClearTextQuery();
          }}
        >
          收起查询
        </button>
      {/if}
    </form>
  </div>

  <!-- 按需展开的单字/词组历史趋势面板（无查询时绝不渲染空图表） -->
  {#if activeQuery}
    <section class="section-block query-result-panel">
      <SectionHeader
        title="历史趋势查询：「{activeQuery}」"
        subtitle={activeQueryTable === "word_freq" ? "词组 (n-gram) 每日出现频次" : "单字/符号每日出现频次"}
      />
      <TrendChart chartSpec={queryTrendSpec} height="175px" />
    </section>
  {/if}

  <!-- 第二部分：高频汉字与高频词组（紧凑双栏比例条列表，彻底杜绝少量数据时的超粗柱状图） -->
  <div class="split-equal">
    <section class="section-block">
      <SectionHeader
        title="高频汉字排行"
        subtitle="点击任意汉字可展开其每日输入趋势"
        meta="显示前 {Math.min(20, hanCharItems.length)} 项"
      />
      {#if hanCharItems.length === 0}
        <EmptyState
          title="尚无高频汉字记录"
          description={textStatsOn ? "通过输入法或编辑器上屏中文后将自动统计字频（仅存计数，不存原文）" : "字频统计默认关闭以保护隐私，您可在上方或设置页随时开启"}
          compact={true}
        />
      {:else}
        <RankingList
          items={hanCharItems}
          total={han || totalChars}
          columns={2}
          maxItems={20}
          selectedKey={activeQueryTable === "char_freq" ? activeQuery : null}
          onSelect={(item) => selectFromList("char_freq", item)}
        />
      {/if}
    </section>

    <section class="section-block">
      <SectionHeader
        title="高频词组 (n-gram)"
        subtitle="基于单次上屏文本切分的二字/三字高频词"
        meta="显示前 {Math.min(20, wordItems.length)} 项"
      />
      {#if wordItems.length === 0}
        <EmptyState
          title="尚无高频词组记录"
          description={segmentOn ? "当单次上屏满 2 个及以上汉字时，将自动提取双字与三字词组频次" : "开启「字频统计」与「n-gram 词组」后即可统计常用词组"}
          compact={true}
        />
      {:else}
        <RankingList
          items={wordItems}
          columns={2}
          maxItems={20}
          barColor="var(--cat-latin)"
          selectedKey={activeQueryTable === "word_freq" ? activeQuery : null}
          onSelect={(item) => selectFromList("word_freq", item)}
        />
      {/if}
    </section>
  </div>

  <!-- 第三部分：字符构成比例与常用标点符号明细 -->
  <div class="split-equal hairline-top">
    <section class="section-block">
      <SectionHeader
        title="字符类别构成比例"
        subtitle="汉字、英文、数字与标点符号分布"
        meta="总计 {fmtNum(totalChars)} 字符"
      />
      <DistributionChart mode="proportion" segments={compositionSegments} />
    </section>

    <section class="section-block">
      <SectionHeader
        title="常用标点符号统计"
        subtitle="独立归类的全角/半角标点频次（不混入高频汉字）"
        meta="共 {punctCharItems.length} 种已记录标点"
      />
      <RankingList
        items={punctCharItems}
        total={punct}
        columns={2}
        maxItems={10}
        barColor="var(--cat-punct)"
        selectedKey={activeQueryTable === "char_freq" ? activeQuery : null}
        onSelect={(item) => selectFromList("char_freq", item)}
        emptyText="所选范围内暂无标点符号频次记录"
      />
    </section>
  </div>
</div>

<style>
  .toolbar-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--sp-3);
    padding: 8px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-tag {
    font-size: var(--text-xs);
    padding: 2px 8px;
    border-radius: var(--radius-xs);
    background: var(--bg-subtle);
    color: var(--text-muted);
    font-weight: 500;
  }

  .status-tag.on {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .query-form {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .query-select {
    height: 28px;
    padding: 0 8px;
    font-size: var(--text-xs);
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
  }

  .query-input {
    width: 240px;
    height: 28px;
    padding: 0 10px;
    font-size: var(--text-xs);
    background: var(--bg-app);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
  }

  .query-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .query-result-panel {
    background: var(--bg-surface);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-md);
    padding: 12px 16px;
  }
</style>
