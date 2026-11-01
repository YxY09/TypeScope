<script>
  import { FULL_LAYOUT } from "../lib/keys.js";
  import { fmtNum, fmtDay, WEEKDAY_NAMES, addDays, weekdayMon } from "../lib/format.js";

  // Supports 3 heatmap modes:
  // - "keyboard": Full 104-key flat ANSI keyboard heatmap (NO 3D keycaps!)
  // - "calendar": 26-week GitHub/Linear style daily activity calendar grid
  // - "hour-grid": 7 weekdays × 24 hours activity density matrix
  export let mode = "keyboard";

  // Keyboard props
  export let keyCounts = []; // [{ vk, n }]
  export let selectedVk = null;
  export let onSelectKey = null;

  // Calendar props
  export let calendarDays = []; // [{ day, physical, active_ms }]
  export let today = 0;

  // Hour-grid props
  export let hourGrid = []; // 7 x 24 matrix

  function heatLevel(val, maxVal) {
    if (!val || val <= 0 || !maxVal || maxVal <= 0) return 0;
    const ratio = val / maxVal;
    if (ratio < 0.15) return 1;
    if (ratio < 0.4) return 2;
    if (ratio < 0.7) return 3;
    return 4;
  }

  // Keyboard reactive maps
  $: keyMap = new Map((keyCounts || []).map((k) => [Number(k.vk), Number(k.n) || 0]));
  $: maxKeyCount = Math.max(1, ...Array.from(keyMap.values()));

  // Calendar reactive cells (26 weeks = 182 days ending at `today`)
  $: calendarWeeks = (() => {
    if (mode !== "calendar" || !today) return [];
    const dayMap = new Map((calendarDays || []).map((d) => [Number(d.day), Number(d.physical) || 0]));
    const maxCal = Math.max(1, ...Array.from(dayMap.values()));
    const endDow = weekdayMon(today); // 0=Mon .. 6=Sun
    const lastMonday = addDays(today, -endDow);
    const startMonday = addDays(lastMonday, -25 * 7);
    const weeks = [];
    for (let w = 0; w < 26; w++) {
      const weekMonday = addDays(startMonday, w * 7);
      const cells = [];
      for (let dow = 0; dow < 7; dow++) {
        const d = addDays(weekMonday, dow);
        const val = dayMap.get(d) || 0;
        const future = d > today;
        cells.push({
          day: d,
          val,
          future,
          level: future ? 0 : heatLevel(val, maxCal)
        });
      }
      const mStr = String(weekMonday).slice(4, 6);
      const prevMStr = w > 0 ? String(addDays(startMonday, (w - 1) * 7)).slice(4, 6) : "";
      weeks.push({
        monthLabel: mStr !== prevMStr ? `${Number(mStr)}月` : "",
        cells
      });
    }
    return weeks;
  })();

  // Hour-grid reactive matrix (7 x 24)
  $: maxHourCell = (() => {
    if (mode !== "hour-grid" || !Array.isArray(hourGrid)) return 1;
    let m = 1;
    for (const row of hourGrid) {
      if (Array.isArray(row)) {
        for (const v of row) {
          if (Number(v) > m) m = Number(v);
        }
      }
    }
    return m;
  })();
</script>

{#if mode === "keyboard"}
  <div class="keyboard-heatmap-wrap">
    <div class="keyboard-stage">
      {#each FULL_LAYOUT as k}
        {@const count = keyMap.get(k.vk) || 0}
        {@const level = heatLevel(count, maxKeyCount)}
        <button
          type="button"
          class="flat-key heat-{level}"
          class:selected={selectedVk === k.vk}
          style="
            left: calc({k.col} * (100% / 23));
            top: calc({k.row} * (100% / 6));
            width: calc({k.w || 1} * (100% / 23) - 3px);
            height: calc({k.h || 1} * (100% / 6) - 3px);
          "
          title="{k.label}: {fmtNum(count)} 次"
          on:click={() => onSelectKey && onSelectKey(k.vk)}
        >
          <span class="key-cap-label">{k.label}</span>
          {#if count > 0}
            <span class="key-cap-count num">{count > 9999 ? `${(count / 1000).toFixed(1)}k` : count}</span>
          {/if}
        </button>
      {/each}
    </div>

    <div class="heatmap-legend">
      <span class="legend-hint">点击任意键位查看单键历史趋势</span>
      <div class="ramp">
        <span>低</span>
        <i class="cell heat-0"></i>
        <i class="cell heat-1"></i>
        <i class="cell heat-2"></i>
        <i class="cell heat-3"></i>
        <i class="cell heat-4"></i>
        <span>高</span>
      </div>
    </div>
  </div>
{:else if mode === "calendar"}
  <div class="calendar-heatmap">
    <div class="cal-grid-wrap">
      <div class="cal-dow-col">
        <span>周一</span>
        <span>周三</span>
        <span>周五</span>
        <span>周日</span>
      </div>
      <div class="cal-weeks">
        {#each calendarWeeks as wk}
          <div class="cal-week-col">
            <span class="cal-month-tag">{wk.monthLabel}</span>
            {#each wk.cells as cell}
              <div
                class="cal-cell heat-{cell.level}"
                class:future={cell.future}
                title={cell.future ? fmtDay(cell.day) : `${fmtDay(cell.day)}: ${fmtNum(cell.val)} 次按键`}
              ></div>
            {/each}
          </div>
        {/each}
      </div>
    </div>

    <div class="heatmap-legend">
      <span class="legend-hint">近 26 周每日物理按键活跃度</span>
      <div class="ramp">
        <span>0</span>
        <i class="cell heat-0"></i>
        <i class="cell heat-1"></i>
        <i class="cell heat-2"></i>
        <i class="cell heat-3"></i>
        <i class="cell heat-4"></i>
        <span>高频</span>
      </div>
    </div>
  </div>
{:else if mode === "hour-grid"}
  <div class="hour-grid-heatmap">
    <div class="hg-header">
      <span class="hg-corner"></span>
      {#each Array.from({ length: 24 }, (_, h) => h) as h}
        <span class="hg-hour mono">{h % 2 === 0 ? String(h).padStart(2, "0") : ""}</span>
      {/each}
    </div>
    {#each WEEKDAY_NAMES as wName, dow}
      <div class="hg-row">
        <span class="hg-dow">{wName}</span>
        {#each Array.from({ length: 24 }, (_, h) => h) as h}
          {@const val = Number(hourGrid?.[dow]?.[h] || 0)}
          {@const level = heatLevel(val, maxHourCell)}
          <div
            class="hg-cell heat-{level}"
            title="{wName} {String(h).padStart(2, '0')}:00 — {fmtNum(val)} 次按键"
          ></div>
        {/each}
      </div>
    {/each}
  </div>
{/if}

<style>
  /* Shared Monochrome Heat Levels */
  .heat-0 {
    background-color: var(--heat-0);
    color: var(--text-muted);
  }
  .heat-1 {
    background-color: var(--heat-1);
    color: var(--text-primary);
  }
  .heat-2 {
    background-color: var(--heat-2);
    color: #1f1f1f;
  }
  .heat-3 {
    background-color: var(--heat-3);
    color: #ffffff;
  }
  .heat-4 {
    background-color: var(--heat-4);
    color: #ffffff;
  }

  :global(:root[data-theme="dark"]) .heat-1,
  :global(:root[data-theme="dark"]) .heat-2 {
    color: #f5f5f5;
  }

  :global(:root[data-theme="dark"]) .heat-3,
  :global(:root[data-theme="dark"]) .heat-4 {
    color: #141414;
  }

  /* 1. Flat 104-Key ANSI Keyboard Heatmap */
  .keyboard-heatmap-wrap {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .keyboard-stage {
    position: relative;
    width: 100%;
    height: 204px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 8px;
  }

  .flat-key {
    position: absolute;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-xs);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: space-between;
    padding: 3px 4px;
    text-align: left;
    overflow: hidden;
    transition: border-color 0.1s ease;
  }

  .flat-key:hover {
    border-color: var(--accent);
    z-index: 2;
  }

  .flat-key.selected {
    outline: 2px solid var(--accent);
    outline-offset: 0px;
    z-index: 3;
  }

  .key-cap-label {
    font-size: 10px;
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
  }

  .key-cap-count {
    font-size: 9px;
    line-height: 1;
    align-self: flex-end;
    opacity: 0.9;
  }

  .heatmap-legend {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .ramp {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .ramp .cell {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    display: inline-block;
    border: 1px solid var(--border-subtle);
  }

  /* 2. Calendar 26-Week Heatmap */
  .calendar-heatmap {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .cal-grid-wrap {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 2px;
  }

  .cal-dow-col {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 102px;
    font-size: 10px;
    color: var(--text-muted);
    padding: 2px 0;
    flex-shrink: 0;
  }

  .cal-weeks {
    display: grid;
    grid-template-columns: repeat(26, minmax(12px, 1fr));
    gap: 3px;
    flex: 1;
  }

  .cal-week-col {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .cal-month-tag {
    height: 14px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .cal-cell {
    width: 100%;
    aspect-ratio: 1 / 1;
    min-height: 11px;
    border-radius: 2px;
    border: 1px solid rgba(0, 0, 0, 0.03);
  }

  .cal-cell.future {
    opacity: 0.25;
  }

  /* 3. 7x24 Hour Grid Heatmap */
  .hour-grid-heatmap {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .hg-header,
  .hg-row {
    display: grid;
    grid-template-columns: 36px repeat(24, minmax(0, 1fr));
    gap: 3px;
    align-items: center;
  }

  .hg-hour {
    font-size: 10px;
    color: var(--text-muted);
    text-align: center;
  }

  .hg-dow {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .hg-cell {
    height: 16px;
    border-radius: 2px;
  }
</style>
