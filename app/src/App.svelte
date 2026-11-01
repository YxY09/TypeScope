<script>
  import { onDestroy, onMount } from "svelte";
  import AppShell from "./components/AppShell.svelte";
  import Overview from "./pages/Overview.svelte";
  import Analytics from "./pages/Analytics.svelte";
  import Text from "./pages/Text.svelte";
  import Settings from "./pages/Settings.svelte";
  import {
    fetchSnapshot,
    fetchRangeReport,
    fetchKeyTrend,
    fetchTextTrend,
    setFlag,
    setIdle,
    sendCommand,
    setAutostart,
    openExclude,
    openDataDir,
    spawnAgent,
    installRime
  } from "./lib/api.js";
  import {
    todayDay,
    addDays,
    diffDays,
    rangeForPreset,
    fillDays,
    fmtDay
  } from "./lib/format.js";

  const THEME_STORAGE_KEY = "typescope_theme_v4";

  let activeTab = "overview";
  let analyticsSubTab = "speed";
  let preset = "today";

  let effectiveToday = todayDay();
  let customStart = addDays(effectiveToday, -6);
  let customEnd = effectiveToday;
  let customStartInput = fmtDay(customStart);
  let customEndInput = fmtDay(customEnd);

  let snap = null;
  let rangeReport = null;
  let rangeDays = [];
  let prevRangeDays = [];
  let rangeTotals = {};
  let prevRangeTotals = {};

  let selectedVk = 32; // Default Space key
  let keyTrendPoints = [];

  let activeQuery = "";
  let activeQueryTable = "char_freq";
  let textTrendPoints = [];

  let themeMode = "light"; // 'light' | 'dark' | 'system'
  let resolvedTheme = "light";

  let busy = false;
  let errorMessage = "";
  let rimeNotice = "";
  let pollTimer = null;
  let mediaQuery = null;
  let mediaChangeHandler = null;
  let visibilityHandler = null;

  $: currentRange = rangeForPreset(preset, customStart, customEnd, effectiveToday);
  $: isSingleDay = currentRange.start === currentRange.end && currentRange.end === effectiveToday;

  function applyTheme(mode) {
    themeMode = mode;
    try {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    } catch (_) {}
    if (mode === "system") {
      const prefersDark =
        typeof window !== "undefined" &&
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches;
      resolvedTheme = prefersDark ? "dark" : "light";
    } else {
      resolvedTheme = mode === "dark" ? "dark" : "light";
    }
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", resolvedTheme);
    }
  }

  function sumDays(days = []) {
    return days.reduce(
      (acc, d) => {
        acc.physical += Number(d.physical) || 0;
        acc.active_ms += Number(d.active_ms) || 0;
        acc.backspace += Number(d.backspace) || 0;
        acc.han += Number(d.han) || 0;
        acc.latin += Number(d.latin) || 0;
        acc.digit += Number(d.digit) || 0;
        acc.punct += Number(d.punct) || 0;
        acc.commits += Number(d.commits) || 0;
        acc.sessions += Number(d.sessions) || 0;
        acc.best_10 = Math.max(acc.best_10, Number(d.best_10) || 0);
        return acc;
      },
      {
        physical: 0,
        active_ms: 0,
        backspace: 0,
        han: 0,
        latin: 0,
        digit: 0,
        punct: 0,
        commits: 0,
        sessions: 0,
        best_10: 0
      }
    );
  }

  async function loadRangeData(start, end) {
    try {
      const span = Math.max(1, diffDays(start, end));
      const prevStart = addDays(start, -span);
      const prevEnd = addDays(start, -1);

      const [currRep, fullSpanRep] = await Promise.all([
        fetchRangeReport(start, end),
        fetchRangeReport(prevStart, end)
      ]);

      rangeReport = currRep;
      const currFilled = fillDays(currRep.days || [], start, end);
      const prevFilled = fillDays(fullSpanRep.days || [], prevStart, prevEnd);

      rangeDays = currFilled;
      prevRangeDays = prevFilled;
      rangeTotals = sumDays(currFilled);
      prevRangeTotals = sumDays(prevFilled);

      // Update key trend if keyboard analytics is active or default
      await loadKeyTrend(selectedVk, start, end);
      if (activeQuery) {
        await loadTextTrend(activeQueryTable, activeQuery, start, end);
      }
    } catch (err) {
      errorMessage = `读取区间统计失败: ${err}`;
    }
  }

  async function loadKeyTrend(vk, start = currentRange.start, end = currentRange.end) {
    try {
      // Expand window to at least 7 days so single-day preset still shows historical context
      const effStart = start === end ? addDays(end, -6) : start;
      keyTrendPoints = await fetchKeyTrend(Number(vk), effStart, end);
    } catch (_) {
      keyTrendPoints = [];
    }
  }

  async function loadTextTrend(table, key, start = currentRange.start, end = currentRange.end) {
    try {
      const effStart = start === end ? addDays(end, -6) : start;
      textTrendPoints = await fetchTextTrend(table, key, effStart, end);
    } catch (_) {
      textTrendPoints = [];
    }
  }

  async function refreshAll(forceRange = false) {
    try {
      const s = await fetchSnapshot(null);
      const prevDay = effectiveToday;
      snap = s;
      if (s && s.day) {
        effectiveToday = s.day;
      }
      errorMessage = "";

      if (forceRange || prevDay !== effectiveToday || !rangeReport) {
        const rng = rangeForPreset(preset, customStart, customEnd, effectiveToday);
        await loadRangeData(rng.start, rng.end);
      }
    } catch (err) {
      errorMessage = `读取统计快照失败: ${err}`;
    }
  }

  function handleSelectTab(tab, sub = null) {
    activeTab = tab;
    if (sub) {
      analyticsSubTab = sub;
    }
  }

  async function handleChangePreset(nextPreset) {
    preset = nextPreset;
    const rng = rangeForPreset(nextPreset, customStart, customEnd, effectiveToday);
    await loadRangeData(rng.start, rng.end);
  }

  async function handleChangeCustom(startStr, endStr) {
    customStartInput = startStr;
    customEndInput = endStr;
    const sNum = Number(String(startStr).replaceAll("-", ""));
    const eNum = Number(String(endStr).replaceAll("-", ""));
    if (sNum > 20000101 && eNum > 20000101) {
      customStart = Math.min(sNum, eNum);
      customEnd = Math.max(sNum, eNum);
      await loadRangeData(customStart, customEnd);
    }
  }

  async function handleSelectKey(vk) {
    selectedVk = Number(vk);
    await loadKeyTrend(selectedVk, currentRange.start, currentRange.end);
  }

  async function handleQueryTextTrend(table, query) {
    activeQueryTable = table;
    activeQuery = query;
    await loadTextTrend(table, query, currentRange.start, currentRange.end);
  }

  function handleClearTextQuery() {
    activeQuery = "";
    textTrendPoints = [];
  }

  async function handleTogglePause() {
    if (!snap || busy) return;
    busy = true;
    try {
      const nextPaused = !snap.flags?.paused;
      await sendCommand(nextPaused ? "pause" : "resume");
      await refreshAll(false);
    } catch (err) {
      errorMessage = `切换采集状态失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleStartAgent() {
    if (busy) return;
    busy = true;
    try {
      await spawnAgent();
      await refreshAll(true);
    } catch (err) {
      errorMessage = `启动后台采集进程失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleToggleFlag(key, boolVal) {
    if (busy) return;
    busy = true;
    try {
      await setFlag(key, boolVal ? "1" : "0");
      await refreshAll(true);
    } catch (err) {
      errorMessage = `更新设置项失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleChangeIdle(ms) {
    if (busy) return;
    busy = true;
    try {
      await setIdle(ms);
      await refreshAll(false);
    } catch (err) {
      errorMessage = `更新空闲阈值失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleToggleAutostart(on) {
    if (busy) return;
    busy = true;
    try {
      await setAutostart(on);
      await refreshAll(false);
    } catch (err) {
      errorMessage = `设置开机自启失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleClearAllData() {
    if (busy) return;
    busy = true;
    try {
      await sendCommand("clear");
      activeQuery = "";
      textTrendPoints = [];
      await refreshAll(true);
    } catch (err) {
      errorMessage = `清空数据失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function handleInstallRime() {
    if (busy) return;
    busy = true;
    errorMessage = "";
    try {
      const msg = await installRime();
      rimeNotice = msg;
      await refreshAll(false);
    } catch (err) {
      errorMessage = `小狼毫配置失败: ${err}`;
    } finally {
      busy = false;
    }
  }

  onMount(async () => {
    let savedTheme = "light";
    try {
      savedTheme = localStorage.getItem(THEME_STORAGE_KEY) || "light";
    } catch (_) {}
    applyTheme(savedTheme);

    if (typeof window !== "undefined" && window.matchMedia) {
      mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      mediaChangeHandler = () => {
        if (themeMode === "system") {
          applyTheme("system");
        }
      };
      mediaQuery.addEventListener("change", mediaChangeHandler);
    }

    if (typeof document !== "undefined") {
      visibilityHandler = () => {
        if (!document.hidden) {
          refreshAll(false);
        }
      };
      document.addEventListener("visibilitychange", visibilityHandler);
    }

    await refreshAll(true);
    pollTimer = setInterval(() => {
      if (typeof document !== "undefined" && document.hidden) {
        return;
      }
      refreshAll(false);
    }, 2000);
  });

  onDestroy(() => {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
    if (mediaQuery && mediaChangeHandler) {
      mediaQuery.removeEventListener("change", mediaChangeHandler);
      mediaChangeHandler = null;
    }
    if (typeof document !== "undefined" && visibilityHandler) {
      document.removeEventListener("visibilitychange", visibilityHandler);
      visibilityHandler = null;
    }
  });
</script>

<AppShell
  {activeTab}
  {preset}
  {customStartInput}
  {customEndInput}
  {currentRange}
  agentOn={Boolean(snap?.agent_on ?? true)}
  paused={Boolean(snap?.flags?.paused)}
  {busy}
  {errorMessage}
  onSelectTab={(tab) => handleSelectTab(tab)}
  onChangePreset={handleChangePreset}
  onChangeCustom={handleChangeCustom}
  onTogglePause={handleTogglePause}
  onStartAgent={handleStartAgent}
>
  {#if activeTab === "overview"}
    <Overview
      {snap}
      {rangeDays}
      {prevRangeDays}
      {rangeTotals}
      {prevRangeTotals}
      {rangeReport}
      {isSingleDay}
      theme={resolvedTheme}
      onNavigate={handleSelectTab}
    />
  {:else if activeTab === "analytics"}
    <Analytics
      subTab={analyticsSubTab}
      {snap}
      {rangeDays}
      {rangeTotals}
      {rangeReport}
      {currentRange}
      {isSingleDay}
      theme={resolvedTheme}
      {selectedVk}
      {keyTrendPoints}
      onChangeSubTab={(sub) => (analyticsSubTab = sub)}
      onSelectKey={handleSelectKey}
      onEnableAppStats={() => handleToggleFlag("app_stats", true)}
      onOpenExclude={() => openExclude()}
    />
  {:else if activeTab === "text"}
    <Text
      {snap}
      {rangeTotals}
      {rangeReport}
      {currentRange}
      {isSingleDay}
      theme={resolvedTheme}
      {activeQuery}
      {activeQueryTable}
      {textTrendPoints}
      onQueryTextTrend={handleQueryTextTrend}
      onClearTextQuery={handleClearTextQuery}
      onToggleFlag={handleToggleFlag}
    />
  {:else if activeTab === "settings"}
    <Settings
      {snap}
      {themeMode}
      {busy}
      {rimeNotice}
      onChangeThemeMode={(m) => applyTheme(m)}
      onTogglePause={handleTogglePause}
      onStartAgent={handleStartAgent}
      onChangeIdle={handleChangeIdle}
      onToggleAutostart={handleToggleAutostart}
      onToggleFlag={handleToggleFlag}
      onOpenExclude={() => openExclude()}
      onOpenDataDir={() => openDataDir()}
      onClearAllData={handleClearAllData}
      onInstallRime={handleInstallRime}
    />
  {/if}
</AppShell>
