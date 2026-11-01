<script>
  export let agentOn = true;
  export let paused = false;
  export let busy = false;
  export let onTogglePause = () => {};
  export let onStartAgent = () => {};

  $: stateClass = !agentOn ? "error" : paused ? "warn" : "ok";
  $: stateLabel = !agentOn ? "采集中断" : paused ? "已暂停" : "记录中";
</script>

<div class="status-cluster">
  <div class="status-pill {stateClass}" title={!agentOn ? "后台采集进程未运行" : paused ? "采集已暂停" : "后台采集正常运行"}>
    <span class="dot"></span>
    <span class="label">{stateLabel}</span>
  </div>

  {#if !agentOn}
    <button type="button" class="action-btn primary" disabled={busy} on:click={onStartAgent}>
      启动采集
    </button>
  {:else}
    <button type="button" class="action-btn" disabled={busy} on:click={onTogglePause}>
      {paused ? "恢复" : "暂停"}
    </button>
  {/if}
</div>

<style>
  .status-cluster {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 8px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-weight: 500;
    border: 1px solid transparent;
  }

  .status-pill .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .status-pill.ok {
    color: var(--status-ok);
    background: var(--status-ok-bg);
  }
  .status-pill.ok .dot {
    background: var(--status-ok);
  }

  .status-pill.warn {
    color: var(--status-warn);
    background: var(--status-warn-bg);
  }
  .status-pill.warn .dot {
    background: var(--status-warn);
  }

  .status-pill.error {
    color: var(--status-danger);
    background: var(--status-danger-bg);
  }
  .status-pill.error .dot {
    background: var(--status-danger);
  }

  .action-btn {
    height: 24px;
    padding: 0 9px;
    font-size: var(--text-xs);
    font-weight: 500;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-secondary);
    transition: all 0.12s ease;
  }

  .action-btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .action-btn.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }
</style>
