<script>
  import Navigation from "./Navigation.svelte";
  import DateRangePicker from "./DateRangePicker.svelte";
  import StatusIndicator from "./StatusIndicator.svelte";
  import { windowControl } from "../lib/api.js";

  export let activeTab = "overview";
  export let preset = "today";
  export let customStartInput = "";
  export let customEndInput = "";
  export let currentRange = { start: 0, end: 0 };
  export let agentOn = true;
  export let paused = false;
  export let busy = false;
  export let errorMessage = "";

  export let onSelectTab = () => {};
  export let onChangePreset = () => {};
  export let onChangeCustom = () => {};
  export let onTogglePause = () => {};
  export let onStartAgent = () => {};

  let isMaximized = false;

  function isInteractiveTarget(target) {
    if (!target || typeof target.closest !== "function") return false;
    return Boolean(target.closest("button, input, select, a, [role='button']"));
  }

  function handleHeaderMouseDown(e) {
    if (e.button !== 0 || isInteractiveTarget(e.target)) return;
    windowControl("drag").catch(() => {});
  }

  async function handleHeaderDblClick(e) {
    if (isInteractiveTarget(e.target)) return;
    try {
      isMaximized = await windowControl("toggle_max");
    } catch (_) {}
  }

  async function handleMinimize() {
    try {
      await windowControl("minimize");
    } catch (_) {}
  }

  async function handleToggleMax() {
    try {
      isMaximized = await windowControl("toggle_max");
    } catch (_) {}
  }

  async function handleClose() {
    try {
      await windowControl("close");
    } catch (_) {}
  }
</script>

<div class="app-shell">
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <header
    class="topbar"
    on:mousedown={handleHeaderMouseDown}
    on:dblclick={handleHeaderDblClick}
  >
    <div class="topbar-inner">
      <div class="left-group">
        <div class="brand">
          <svg class="brand-mark" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
            <rect class="mark-bg" x="1" y="1" width="22" height="22" rx="5" />
            <rect class="mark-bar-low" x="6" y="10" width="3" height="4.5" rx="0.8" />
            <rect class="mark-bar-mid" x="10.5" y="7.5" width="3" height="7" rx="0.8" />
            <rect class="mark-bar-high" x="15" y="5" width="3" height="9.5" rx="0.8" />
            <rect class="mark-space" x="6" y="16.2" width="12" height="2.6" rx="0.8" />
          </svg>
          <span class="brand-name">TypeScope</span>
        </div>
        <div class="divider-v"></div>
        <Navigation {activeTab} onSelect={onSelectTab} />
      </div>

      <div class="right-group">
        {#if activeTab !== "settings"}
          <DateRangePicker
            {preset}
            {customStartInput}
            {customEndInput}
            {currentRange}
            {onChangePreset}
            {onChangeCustom}
          />
          <div class="divider-v"></div>
        {/if}
        <StatusIndicator
          {agentOn}
          {paused}
          {busy}
          {onTogglePause}
          {onStartAgent}
        />
        <div class="divider-v"></div>
        <div class="win-controls" aria-label="窗口控制">
          <button
            type="button"
            class="win-btn"
            title="最小化"
            aria-label="最小化"
            on:click={handleMinimize}
          >
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M1 5h8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
          </button>
          <button
            type="button"
            class="win-btn"
            title={isMaximized ? "还原" : "最大化"}
            aria-label={isMaximized ? "还原" : "最大化"}
            on:click={handleToggleMax}
          >
            {#if isMaximized}
              <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                <rect x="1.5" y="3" width="5.5" height="5.5" fill="none" stroke="currentColor" stroke-width="1.2" />
                <path d="M3 3V1.5h5.5V7H7" fill="none" stroke="currentColor" stroke-width="1.2" />
              </svg>
            {:else}
              <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                <rect x="1.5" y="1.5" width="7" height="7" rx="0.8" fill="none" stroke="currentColor" stroke-width="1.2" />
              </svg>
            {/if}
          </button>
          <button
            type="button"
            class="win-btn win-btn-close"
            title="关闭窗口"
            aria-label="关闭窗口"
            on:click={handleClose}
          >
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M1.8 1.8l6.4 6.4M8.2 1.8l-6.4 6.4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </header>

  {#if errorMessage}
    <div class="error-banner" role="alert">
      <span>{errorMessage}</span>
    </div>
  {/if}

  <main class="viewport">
    <slot />
  </main>
</div>

<style>
  .app-shell {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-app);
    border: 1px solid var(--border-default);
  }

  .topbar {
    flex-shrink: 0;
    height: 46px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    padding: 0 var(--sp-4) 0 var(--sp-6);
  }

  .topbar-inner {
    max-width: 1400px;
    height: 100%;
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
  }

  .left-group,
  .right-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .brand-mark {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: block;
  }

  .brand-mark .mark-bg {
    fill: var(--text-primary);
  }

  .brand-mark .mark-bar-low {
    fill: var(--bg-surface);
    opacity: 0.55;
  }

  .brand-mark .mark-bar-mid {
    fill: var(--bg-surface);
    opacity: 0.78;
  }

  .brand-mark .mark-bar-high,
  .brand-mark .mark-space {
    fill: var(--bg-surface);
  }

  .brand-name {
    font-size: var(--text-md);
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-primary);
  }

  .divider-v {
    width: 1px;
    height: 16px;
    background: var(--border-default);
  }

  .win-controls {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .win-btn {
    width: 28px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: transparent;
    transition: background-color 0.12s ease, color 0.12s ease;
  }

  .win-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .win-btn-close:hover {
    background: #DC2626;
    color: #FFFFFF;
  }

  .error-banner {
    flex-shrink: 0;
    padding: 6px var(--sp-6);
    background: var(--status-danger-bg);
    border-bottom: 1px solid rgba(220, 38, 38, 0.25);
    color: var(--status-danger);
    font-size: var(--text-sm);
    text-align: center;
  }

  .viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  @media (max-width: 1120px) {
    .topbar {
      padding: 0 var(--sp-3) 0 var(--sp-4);
    }
  }
</style>
