import { fmtShort, fmtDay, fmtNum, kpm, INTERVAL_LABELS, addDays } from "./format.js";

export function getChartPalette(theme = "light") {
  const isDark = theme === "dark";
  return {
    isDark,
    textPrimary: isDark ? "#f5f5f5" : "#1f1f1f",
    textSecondary: isDark ? "#a3a3a3" : "#737373",
    textMuted: isDark ? "#737373" : "#a3a3a3",
    axisLine: isDark ? "#303030" : "#e5e5e5",
    splitLine: isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.06)",
    tooltipBg: isDark ? "#1c1c1c" : "#ffffff",
    tooltipBorder: isDark ? "#3a3a3a" : "#dcdcdc",
    accent: isDark ? "#f5f5f5" : "#171717",
    accentSoft: isDark ? "rgba(245, 245, 245, 0.18)" : "rgba(23, 23, 23, 0.12)",
    accentAreaTop: isDark ? "rgba(245, 245, 245, 0.18)" : "rgba(23, 23, 23, 0.12)",
    accentAreaBottom: isDark ? "rgba(245, 245, 245, 0.0)" : "rgba(23, 23, 23, 0.0)",
    secondary: isDark ? "#9e9e9e" : "#666666",
    secondarySoft: isDark ? "rgba(158, 158, 158, 0.2)" : "rgba(102, 102, 102, 0.14)",
    compareLine: isDark ? "#737373" : "#999999"
  };
}

function baseTooltip(p, extra = {}) {
  return {
    trigger: "axis",
    confine: true,
    backgroundColor: p.tooltipBg,
    borderColor: p.tooltipBorder,
    borderWidth: 1,
    padding: [8, 10],
    textStyle: {
      color: p.textPrimary,
      fontSize: 12,
      fontFamily: '"JetBrains Mono", "Segoe UI", sans-serif'
    },
    axisPointer: {
      type: "line",
      lineStyle: { color: p.axisLine, width: 1, type: "solid" }
    },
    ...extra
  };
}

function baseGrid(overrides = {}) {
  return {
    top: 26,
    right: 14,
    bottom: 24,
    left: 44,
    containLabel: false,
    ...overrides
  };
}

export function inspectSeriesDensity(values = []) {
  const arr = Array.isArray(values) ? values : [];
  const nonZeroCount = arr.filter((v) => Number(v) > 0).length;
  if (arr.length === 0 || nonZeroCount === 0) {
    return { state: "empty", total: arr.length, nonZeroCount: 0 };
  }
  if (arr.length === 1) {
    return { state: "single", total: 1, nonZeroCount };
  }
  if (arr.length <= 3) {
    return { state: "sparse", total: arr.length, nonZeroCount };
  }
  return { state: "normal", total: arr.length, nonZeroCount };
}

export function buildOverviewTrendOption({
  isSingleDay,
  hours = [],
  days = [],
  prevDays = [],
  theme = "light"
}) {
  const p = getChartPalette(theme);

  if (isSingleDay) {
    const physicalSeries = Array.from({ length: 24 }, (_, h) => hours[h]?.physical || 0);
    const commitSeries = Array.from({ length: 24 }, (_, h) => (hours[h]?.han || 0) + (hours[h]?.latin || 0));
    const density = inspectSeriesDensity(physicalSeries);
    if (density.state === "empty") {
      return { empty: true, reason: "今日尚无按键记录" };
    }

    const labels = Array.from({ length: 24 }, (_, h) => `${String(h).padStart(2, "0")}:00`);
    return {
      empty: false,
      option: {
        animation: false,
        grid: baseGrid({ top: 28, left: 46, right: 12, bottom: 24 }),
        legend: {
          top: 0,
          right: 4,
          itemWidth: 10,
          itemHeight: 6,
          textStyle: { color: p.textSecondary, fontSize: 11 },
          data: ["物理按键", "中英字符"]
        },
        tooltip: baseTooltip(p, {
          axisPointer: { type: "shadow", shadowStyle: { color: p.isDark ? "rgba(255,255,255,0.03)" : "rgba(0,0,0,0.03)" } },
          formatter(params) {
            if (!params || !params.length) return "";
            const title = params[0].axisValueLabel;
            const lines = [`<div style="font-weight:600;margin-bottom:4px">${title}</div>`];
            for (const item of params) {
              lines.push(
                `<div style="display:flex;justify-content:space-between;gap:16px"><span>${item.marker}${item.seriesName}</span><span style="font-weight:600">${fmtNum(item.value)}</span></div>`
              );
            }
            return lines.join("");
          }
        }),
        xAxis: {
          type: "category",
          data: labels,
          axisLine: { lineStyle: { color: p.axisLine } },
          axisTick: { show: false },
          axisLabel: {
            color: p.textSecondary,
            fontSize: 11,
            interval: 3
          }
        },
        yAxis: {
          type: "value",
          splitNumber: 4,
          axisLine: { show: false },
          axisTick: { show: false },
          splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
          axisLabel: { color: p.textSecondary, fontSize: 11 }
        },
        series: [
          {
            name: "物理按键",
            type: "bar",
            data: physicalSeries,
            barMaxWidth: 14,
            itemStyle: {
              color: p.accent,
              borderRadius: [2, 2, 0, 0]
            }
          },
          {
            name: "中英字符",
            type: "line",
            smooth: 0.2,
            symbol: "emptyCircle",
            symbolSize: 5,
            data: commitSeries,
            itemStyle: { color: p.secondary },
            lineStyle: { width: 1.5, type: "dashed", color: p.secondary }
          }
        ]
      }
    };
  }

  const physicalSeries = days.map((d) => d.physical || 0);
  const density = inspectSeriesDensity(physicalSeries);
  if (density.state === "empty") {
    return { empty: true, reason: "所选日期范围内无输入记录" };
  }

  const labels = days.map((d) => fmtShort(d.day));
  const prevSeries = days.map((_, i) => (prevDays[i] ? prevDays[i].physical || 0 : 0));
  const hasPrev = prevSeries.some((v) => v > 0);
  const useLine = days.length > 21;

  return {
    empty: false,
    option: {
      animation: false,
      grid: baseGrid({ top: 28, left: 48, right: 12, bottom: 24 }),
      legend: {
        top: 0,
        right: 4,
        itemWidth: 12,
        itemHeight: 6,
        textStyle: { color: p.textSecondary, fontSize: 11 },
        data: hasPrev ? ["本期按键", "上期同跨度"] : ["本期按键"]
      },
      tooltip: baseTooltip(p, {
        formatter(params) {
          if (!params || !params.length) return "";
          const idx = params[0].dataIndex;
          const dayObj = days[idx];
          const prevObj = prevDays[idx];
          const lines = [
            `<div style="font-weight:600;margin-bottom:4px">${dayObj ? fmtDay(dayObj.day) : params[0].axisValueLabel}</div>`,
            `<div style="display:flex;justify-content:space-between;gap:16px"><span>本期物理按键</span><span style="font-weight:600">${fmtNum(dayObj ? dayObj.physical : 0)}</span></div>`
          ];
          if (dayObj) {
            const chars = (dayObj.han || 0) + (dayObj.latin || 0) + (dayObj.digit || 0) + (dayObj.punct || 0);
            lines.push(
              `<div style="display:flex;justify-content:space-between;gap:16px"><span>本期提交字符</span><span style="font-weight:600">${fmtNum(chars)}</span></div>`
            );
          }
          if (hasPrev && prevObj) {
            lines.push(
              `<div style="display:flex;justify-content:space-between;gap:16px;color:${p.textSecondary}"><span>上期 (${fmtShort(prevObj.day)})</span><span>${fmtNum(prevObj.physical)}</span></div>`
            );
          }
          return lines.join("");
        }
      }),
      xAxis: {
        type: "category",
        boundaryGap: !useLine,
        data: labels,
        axisLine: { lineStyle: { color: p.axisLine } },
        axisTick: { show: false },
        axisLabel: {
          color: p.textSecondary,
          fontSize: 11,
          interval: days.length > 35 ? Math.floor(days.length / 8) : days.length > 14 ? 2 : 0
        }
      },
      yAxis: {
        type: "value",
        splitNumber: 4,
        axisLine: { show: false },
        axisTick: { show: false },
        splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
        axisLabel: { color: p.textSecondary, fontSize: 11 }
      },
      series: [
        useLine
          ? {
              name: "本期按键",
              type: "line",
              smooth: 0.2,
              symbol: days.length > 40 ? "none" : "circle",
              symbolSize: 4,
              data: physicalSeries,
              itemStyle: { color: p.accent },
              lineStyle: { width: 2, color: p.accent },
              areaStyle: {
                color: {
                  type: "linear",
                  x: 0,
                  y: 0,
                  x2: 0,
                  y2: 1,
                  colorStops: [
                    { offset: 0, color: p.accentAreaTop },
                    { offset: 1, color: p.accentAreaBottom }
                  ]
                }
              }
            }
          : {
              name: "本期按键",
              type: "bar",
              data: physicalSeries,
              barMaxWidth: 20,
              itemStyle: {
                color: p.accent,
                borderRadius: [2, 2, 0, 0]
              }
            },
        ...(hasPrev
          ? [
              {
                name: "上期同跨度",
                type: "line",
                smooth: 0.2,
                symbol: "emptyCircle",
                symbolSize: 4,
                data: prevSeries,
                itemStyle: { color: p.compareLine },
                lineStyle: { width: 1.5, type: "dashed", color: p.compareLine }
              }
            ]
          : [])
      ]
    }
  };
}

export function buildHourlyRhythmOption(hours = [], theme = "light") {
  const p = getChartPalette(theme);
  const values = Array.from({ length: 24 }, (_, h) => hours[h]?.physical || 0);
  const density = inspectSeriesDensity(values);
  if (density.state === "empty") {
    return { empty: true, reason: "暂无分时活动数据" };
  }

  const labels = Array.from({ length: 24 }, (_, h) => `${String(h).padStart(2, "0")}`);
  return {
    empty: false,
    option: {
      animation: false,
      grid: baseGrid({ top: 10, left: 36, right: 8, bottom: 20 }),
      tooltip: baseTooltip(p, {
        axisPointer: { type: "shadow", shadowStyle: { color: p.isDark ? "rgba(255,255,255,0.03)" : "rgba(0,0,0,0.03)" } },
        formatter(params) {
          const item = params?.[0];
          if (!item) return "";
          const h = item.dataIndex;
          const activeMs = hours[h]?.active_ms || 0;
          const speed = kpm(item.value, activeMs);
          return [
            `<div style="font-weight:600;margin-bottom:2px">${item.axisValueLabel}:00 - ${item.axisValueLabel}:59</div>`,
            `<div>物理按键: <b>${fmtNum(item.value)}</b></div>`,
            speed ? `<div>平均速度: <b>${speed} KPM</b></div>` : ""
          ].join("");
        }
      }),
      xAxis: {
        type: "category",
        data: labels,
        axisLine: { lineStyle: { color: p.axisLine } },
        axisTick: { show: false },
        axisLabel: { color: p.textSecondary, fontSize: 10, interval: 3 }
      },
      yAxis: {
        type: "value",
        splitNumber: 3,
        axisLine: { show: false },
        axisTick: { show: false },
        splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
        axisLabel: { color: p.textSecondary, fontSize: 10 }
      },
      series: [
        {
          type: "bar",
          data: values,
          barMaxWidth: 10,
          itemStyle: {
            color: p.accent,
            borderRadius: [2, 2, 0, 0]
          }
        }
      ]
    }
  };
}

export function buildSpeedTrendOption({ isSingleDay, hours = [], days = [], theme = "light" }) {
  const p = getChartPalette(theme);

  if (isSingleDay) {
    const hourlyKpm = Array.from({ length: 24 }, (_, h) => {
      const row = hours[h];
      return row ? kpm(row.physical, row.active_ms) || 0 : 0;
    });
    const density = inspectSeriesDensity(hourlyKpm);
    if (density.state === "empty") {
      return { empty: true, reason: "今日暂无有效速度样本" };
    }
    const labels = Array.from({ length: 24 }, (_, h) => `${String(h).padStart(2, "0")}:00`);
    return {
      empty: false,
      option: {
        animation: false,
        grid: baseGrid({ top: 24, left: 42, right: 12, bottom: 24 }),
        tooltip: baseTooltip(p, {
          formatter(params) {
            const item = params?.[0];
            if (!item) return "";
            return `<div style="font-weight:600">${item.axisValueLabel}</div><div>平均速度: <b>${item.value} KPM</b></div>`;
          }
        }),
        xAxis: {
          type: "category",
          data: labels,
          axisLine: { lineStyle: { color: p.axisLine } },
          axisTick: { show: false },
          axisLabel: { color: p.textSecondary, fontSize: 11, interval: 3 }
        },
        yAxis: {
          type: "value",
          splitNumber: 4,
          axisLine: { show: false },
          axisTick: { show: false },
          splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
          axisLabel: { color: p.textSecondary, fontSize: 11 }
        },
        series: [
          {
            name: "每小时均速",
            type: "bar",
            data: hourlyKpm,
            barMaxWidth: 14,
            itemStyle: { color: p.accent, borderRadius: [2, 2, 0, 0] }
          }
        ]
      }
    };
  }

  const avgSeries = days.map((d) => kpm(d.physical, d.active_ms) || 0);
  const peakSeries = days.map((d) => d.best_10 || 0);
  const density = inspectSeriesDensity([...avgSeries, ...peakSeries]);
  if (density.state === "empty") {
    return { empty: true, reason: "所选范围内无速度记录" };
  }

  const labels = days.map((d) => fmtShort(d.day));
  return {
    empty: false,
    option: {
      animation: false,
      grid: baseGrid({ top: 28, left: 44, right: 12, bottom: 24 }),
      legend: {
        top: 0,
        right: 4,
        itemWidth: 10,
        itemHeight: 6,
        textStyle: { color: p.textSecondary, fontSize: 11 },
        data: ["日均速度 (KPM)", "10秒峰值 (KPM)"]
      },
      tooltip: baseTooltip(p),
      xAxis: {
        type: "category",
        data: labels,
        axisLine: { lineStyle: { color: p.axisLine } },
        axisTick: { show: false },
        axisLabel: {
          color: p.textSecondary,
          fontSize: 11,
          interval: days.length > 35 ? Math.floor(days.length / 8) : days.length > 14 ? 2 : 0
        }
      },
      yAxis: {
        type: "value",
        splitNumber: 4,
        axisLine: { show: false },
        axisTick: { show: false },
        splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
        axisLabel: { color: p.textSecondary, fontSize: 11 }
      },
      series: [
        {
          name: "日均速度 (KPM)",
          type: "bar",
          data: avgSeries,
          barMaxWidth: 18,
          itemStyle: { color: p.accent, borderRadius: [2, 2, 0, 0] }
        },
        {
          name: "10秒峰值 (KPM)",
          type: "line",
          smooth: 0.2,
          symbol: days.length > 35 ? "none" : "emptyCircle",
          symbolSize: 5,
          data: peakSeries,
          itemStyle: { color: p.secondary },
          lineStyle: { width: 1.5, type: "dashed", color: p.secondary }
        }
      ]
    }
  };
}

export function buildIntervalDistributionOption(intervals = [], theme = "light") {
  const p = getChartPalette(theme);
  const values = INTERVAL_LABELS.map((_, i) => Number(intervals?.[i] || 0));
  const density = inspectSeriesDensity(values);
  if (density.state === "empty") {
    return { empty: true, reason: "暂无击键间隔样本" };
  }
  const total = values.reduce((a, b) => a + b, 0) || 1;

  return {
    empty: false,
    option: {
      animation: false,
      grid: baseGrid({ top: 18, left: 46, right: 12, bottom: 26 }),
      tooltip: baseTooltip(p, {
        axisPointer: { type: "shadow", shadowStyle: { color: p.isDark ? "rgba(255,255,255,0.03)" : "rgba(0,0,0,0.03)" } },
        formatter(params) {
          const item = params?.[0];
          if (!item) return "";
          const pct = ((item.value * 100) / total).toFixed(1);
          return `<div style="font-weight:600;margin-bottom:2px">间隔 ${item.axisValueLabel}</div><div>次数: <b>${fmtNum(item.value)}</b> (${pct}%)</div>`;
        }
      }),
      xAxis: {
        type: "category",
        data: INTERVAL_LABELS,
        axisLine: { lineStyle: { color: p.axisLine } },
        axisTick: { show: false },
        axisLabel: { color: p.textSecondary, fontSize: 10, interval: 0, rotate: 20 }
      },
      yAxis: {
        type: "value",
        splitNumber: 4,
        axisLine: { show: false },
        axisTick: { show: false },
        splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
        axisLabel: { color: p.textSecondary, fontSize: 10 }
      },
      series: [
        {
          type: "bar",
          data: values,
          barMaxWidth: 22,
          itemStyle: {
            color: p.accent,
            borderRadius: [2, 2, 0, 0]
          }
        }
      ]
    }
  };
}

export function buildItemTrendOption({ points = [], start, end, label = "", theme = "light" }) {
  const p = getChartPalette(theme);
  const map = new Map((points || []).map((pt) => [pt.day, pt.n]));

  let effStart = start;
  let effEnd = end;
  if (effStart === effEnd) {
    effStart = addDays(effEnd, -6);
  } else if (addDays(effStart, 1) === effEnd) {
    effStart = addDays(effEnd, -6);
  }

  const fullDays = [];
  const values = [];
  for (let d = effStart; d <= effEnd; d = addDays(d, 1)) {
    fullDays.push(d);
    values.push(map.get(d) || 0);
  }

  const density = inspectSeriesDensity(values);
  if (density.state === "empty") {
    return { empty: true, reason: `「${label}」在所选范围内无记录` };
  }

  return {
    empty: false,
    option: {
      animation: false,
      grid: baseGrid({ top: 16, left: 40, right: 12, bottom: 22 }),
      tooltip: baseTooltip(p, {
        formatter(params) {
          const item = params?.[0];
          if (!item) return "";
          const d = fullDays[item.dataIndex];
          return `<div style="font-weight:600">${fmtDay(d)}</div><div>${label}: <b>${fmtNum(item.value)}</b> 次</div>`;
        }
      }),
      xAxis: {
        type: "category",
        data: fullDays.map((d) => fmtShort(d)),
        axisLine: { lineStyle: { color: p.axisLine } },
        axisTick: { show: false },
        axisLabel: {
          color: p.textSecondary,
          fontSize: 10,
          interval: fullDays.length > 21 ? Math.floor(fullDays.length / 6) : 0
        }
      },
      yAxis: {
        type: "value",
        splitNumber: 3,
        minInterval: 1,
        axisLine: { show: false },
        axisTick: { show: false },
        splitLine: { lineStyle: { color: p.splitLine, type: "dashed" } },
        axisLabel: { color: p.textSecondary, fontSize: 10 }
      },
      series: [
        {
          name: label,
          type: "bar",
          data: values,
          barMaxWidth: 16,
          itemStyle: {
            color: p.accent,
            borderRadius: [2, 2, 0, 0]
          }
        }
      ]
    }
  };
}
