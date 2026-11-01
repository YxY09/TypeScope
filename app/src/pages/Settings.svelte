<script>
  import { onDestroy } from "svelte";
  import SettingGroup from "../components/SettingGroup.svelte";
  import SettingRow from "../components/SettingRow.svelte";
  import { fmtNum } from "../lib/format.js";

  export let snap = null;
  export let themeMode = "light"; // 'light' | 'dark' | 'system'
  export let busy = false;

  export let onChangeThemeMode = () => {};
  export let onTogglePause = () => {};
  export let onStartAgent = () => {};
  export let onChangeIdle = () => {};
  export let onToggleAutostart = () => {};
  export let onToggleFlag = () => {};
  export let onOpenExclude = () => {};
  export let onOpenDataDir = () => {};
  export let onClearAllData = () => {};
  export let onInstallRime = async () => "";
  export let rimeNotice = "";

  const idleOptions = [
    { ms: 3000, label: "3 秒" },
    { ms: 5000, label: "5 秒 (默认)" },
    { ms: 8000, label: "8 秒" },
    { ms: 10000, label: "10 秒" },
    { ms: 15000, label: "15 秒" }
  ];

  let clearConfirmArmed = false;
  let clearTimer = null;

  function armClearConfirm() {
    clearConfirmArmed = true;
    if (clearTimer) clearTimeout(clearTimer);
    clearTimer = setTimeout(() => {
      clearConfirmArmed = false;
    }, 6000);
  }

  function cancelClearConfirm() {
    clearConfirmArmed = false;
    if (clearTimer) {
      clearTimeout(clearTimer);
      clearTimer = null;
    }
  }

  async function executeClear() {
    cancelClearConfirm();
    await onClearAllData();
  }

  onDestroy(() => {
    if (clearTimer) clearTimeout(clearTimer);
  });

  $: agentOn = Boolean(snap?.agent_on);
  $: paused = Boolean(snap?.flags?.paused);
  $: textStatsOn = Boolean(snap?.flags?.text_stats);
  $: segmentOn = Boolean(snap?.flags?.segment);
  $: appStatsOn = Boolean(snap?.flags?.app_stats);
  $: rimeHello = snap?.flags?.rime_hello || "";
  $: rimeAccepted = Number(snap?.rime_accepted || 0);
  $: idleMs = Number(snap?.idle_ms || 5000);
  $: autostartOn = Boolean(snap?.autostart);
</script>

<div class="page-container settings-container">
  <!-- 1. 外观 -->
  <SettingGroup
    title="外观与显示"
    description="切换界面色彩主题，支持浅色、深色或自动跟随 Windows 系统颜色设置。"
  >
    <SettingRow
      label="界面主题模式"
      description="统一控制所有页面、表格、键盘热力图与图表配色"
    >
      <div class="segmented-control">
        <button
          type="button"
          class="seg-btn"
          class:active={themeMode === "light"}
          on:click={() => onChangeThemeMode("light")}
        >
          浅色
        </button>
        <button
          type="button"
          class="seg-btn"
          class:active={themeMode === "dark"}
          on:click={() => onChangeThemeMode("dark")}
        >
          深色
        </button>
        <button
          type="button"
          class="seg-btn"
          class:active={themeMode === "system"}
          on:click={() => onChangeThemeMode("system")}
        >
          跟随系统
        </button>
      </div>
    </SettingRow>
  </SettingGroup>

  <!-- 2. 输入采集与系统启动 -->
  <SettingGroup
    title="输入采集与系统启动"
    description="管理常驻后台采集进程（Raw Input）、活跃计时判定阈值与开机自启。"
  >
    <SettingRow
      label="后台采集服务状态"
      description="独立轻量级原生进程 typescope-agent.exe，关闭看板窗口不影响后台记录"
      statusText={!agentOn ? "未运行" : paused ? "已暂停" : "运行中"}
      statusTone={!agentOn ? "danger" : paused ? "warn" : "ok"}
    >
      {#if !agentOn}
        <button type="button" class="btn btn-primary" disabled={busy} on:click={onStartAgent}>
          启动采集服务
        </button>
      {:else}
        <button type="button" class="btn" disabled={busy} on:click={onTogglePause}>
          {paused ? "恢复采集" : "暂停采集"}
        </button>
      {/if}
    </SettingRow>

    <SettingRow
      label="有效活跃间隔阈值"
      description="两次物理按键间隔不超过此值时计入有效活跃时间，超时则自动切分会话"
    >
      <select
        class="setting-select"
        value={idleMs}
        disabled={busy}
        on:change={(e) => onChangeIdle(Number(e.currentTarget.value))}
      >
        {#each idleOptions as opt}
          <option value={opt.ms}>{opt.label}</option>
        {/each}
      </select>
    </SettingRow>

    <SettingRow
      label="开机自动启动采集"
      description="登录 Windows 时自动在系统托盘静默运行后台采集进程"
      statusText={autostartOn ? "已启用" : "未启用"}
      statusTone={autostartOn ? "ok" : "neutral"}
    >
      <button
        type="button"
        class="switch-btn"
        class:checked={autostartOn}
        role="switch"
        aria-checked={autostartOn}
        disabled={busy}
        on:click={() => onToggleAutostart(!autostartOn)}
      >
        <span class="switch-thumb"></span>
      </button>
    </SettingRow>
  </SettingGroup>

  <!-- 3. 文字统计与小狼毫集成 -->
  <SettingGroup
    title="文字统计与小狼毫集成"
    description="配置单字频次、二/三字词组切分统计，以及小狼毫 (Weasel / Rime) 本地命名管道连接。"
  >
    <SettingRow
      label="高频单字与标点统计"
      description="将上屏字符拆散为独立单字累加频次，打乱顺序存储，无法还原原始句子"
      statusText={textStatsOn ? "已开启" : "已关闭"}
      statusTone={textStatsOn ? "ok" : "neutral"}
    >
      <button
        type="button"
        class="switch-btn"
        class:checked={textStatsOn}
        role="switch"
        aria-checked={textStatsOn}
        disabled={busy}
        on:click={() => onToggleFlag("text_stats", !textStatsOn)}
      >
        <span class="switch-thumb"></span>
      </button>
    </SettingRow>

    <SettingRow
      label="n-gram 词组切分统计"
      description="在开启字频统计的基础上，对单次上屏文本切分 2~3 字高频词组"
      statusText={segmentOn ? "已开启" : "已关闭"}
      statusTone={segmentOn ? "ok" : "neutral"}
    >
      <button
        type="button"
        class="switch-btn"
        class:checked={segmentOn}
        role="switch"
        aria-checked={segmentOn}
        disabled={busy || !textStatsOn}
        on:click={() => onToggleFlag("segment", !segmentOn)}
      >
        <span class="switch-thumb"></span>
      </button>
    </SettingRow>

    <SettingRow
      label="小狼毫 (Rime) 上屏通道"
      description={rimeNotice || "自动探测本机小狼毫配置目录与输入方案（支持薄荷、雾凇、朙月、双拼等），通过本地共享内存接收上屏事件"}
      statusText={rimeHello || rimeAccepted > 0 ? "已连接" : "等待连接"}
      statusTone={rimeHello || rimeAccepted > 0 ? "ok" : "neutral"}
    >
      <div class="confirm-cluster">
        <span class="meta-readout mono">
          累计接收 {fmtNum(rimeAccepted)} 次提交{rimeHello ? " · 管道已就绪" : ""}
        </span>
        <button type="button" class="btn" disabled={busy} on:click={onInstallRime}>
          {rimeHello || rimeAccepted > 0 ? "重新配置并部署" : "一键接入小狼毫"}
        </button>
      </div>
    </SettingRow>
  </SettingGroup>

  <!-- 4. 程序分类与隐私保护 -->
  <SettingGroup
    title="程序分类与隐私保护"
    description="管理按应用程序分类统计及敏感进程排除列表。所有数据仅保存在本机 SQLite 数据库。"
  >
    <SettingRow
      label="按应用程序分类统计"
      description="开启后记录前台进程文件名（如 Code.exe）的物理按键与活跃时长，不读取窗口标题"
      statusText={appStatsOn ? "已开启" : "已关闭"}
      statusTone={appStatsOn ? "ok" : "neutral"}
    >
      <button
        type="button"
        class="switch-btn"
        class:checked={appStatsOn}
        role="switch"
        aria-checked={appStatsOn}
        disabled={busy}
        on:click={() => onToggleFlag("app_stats", !appStatsOn)}
      >
        <span class="switch-thumb"></span>
      </button>
    </SettingRow>

    <SettingRow
      label="排除程序名单 (exclude.txt)"
      description="名单内的进程（如密码管理器 1Password、KeePass、Bitwarden）在焦点下将自动跳过记录"
    >
      <button type="button" class="btn" on:click={onOpenExclude}>
        编辑排除名单
      </button>
    </SettingRow>
  </SettingGroup>

  <!-- 5. 数据管理 -->
  <SettingGroup
    title="数据管理"
    description="查看本地 SQLite 存储位置或重置所有历史统计数据。"
  >
    <SettingRow
      label="本地数据库目录"
      description="所有统计数据均保存在本机 AppData 目录下的 typescope.sqlite3 文件中"
    >
      <button type="button" class="btn" on:click={onOpenDataDir}>
        打开数据文件夹
      </button>
    </SettingRow>

    <SettingRow
      label="清空全部统计数据"
      description="永久删除本地数据库中的所有按键、速度纪录、字频与日历数据，此操作不可撤销"
    >
      {#if !clearConfirmArmed}
        <button type="button" class="btn btn-danger" disabled={busy} on:click={armClearConfirm}>
          清空全部数据
        </button>
      {:else}
        <div class="confirm-cluster">
          <span class="confirm-warn">确认清空所有历史数据？</span>
          <button type="button" class="btn btn-danger" disabled={busy} on:click={executeClear}>
            确认清空
          </button>
          <button type="button" class="btn" disabled={busy} on:click={cancelClearConfirm}>
            取消
          </button>
        </div>
      {/if}
    </SettingRow>
  </SettingGroup>
</div>

<style>
  .settings-container {
    max-width: 1080px;
    gap: 0;
  }

  .segmented-control {
    display: inline-flex;
    align-items: center;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
  }

  .seg-btn {
    height: 24px;
    padding: 0 12px;
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-xs);
    transition: all 0.12s ease;
  }

  .seg-btn:hover:not(.active) {
    color: var(--text-primary);
  }

  .seg-btn.active {
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
  }

  .setting-select {
    height: 28px;
    padding: 0 10px;
    font-size: var(--text-xs);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
  }

  .switch-btn {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--bg-muted);
    border: 1px solid var(--border-default);
    position: relative;
    padding: 0;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .switch-btn.checked {
    background: var(--accent);
    border-color: var(--accent);
  }

  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-primary);
    transition: transform 0.15s ease, background-color 0.15s ease;
  }

  .switch-btn.checked .switch-thumb {
    transform: translateX(16px);
    background: var(--accent-fg);
  }

  .meta-readout {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    background: var(--bg-subtle);
    padding: 4px 8px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--border-subtle);
  }

  .confirm-cluster {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .confirm-warn {
    font-size: var(--text-xs);
    color: var(--status-danger);
    font-weight: 500;
  }
</style>
