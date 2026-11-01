<script>
  export let label = "";
  export let description = "";
  export let statusText = "";
  export let statusTone = "neutral"; // 'ok' | 'warn' | 'danger' | 'neutral'
</script>

<div class="setting-row">
  <div class="row-info">
    <div class="row-title-line">
      <span class="row-label">{label}</span>
      {#if statusText}
        <span class="row-status {statusTone}">{statusText}</span>
      {/if}
    </div>
    {#if description}
      <p class="row-desc">{description}</p>
    {/if}
  </div>
  <div class="row-control">
    <slot />
  </div>
</div>

<style>
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-5);
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .row-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .row-title-line {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .row-label {
    font-size: var(--text-base);
    font-weight: 500;
    color: var(--text-primary);
  }

  .row-status {
    font-size: var(--text-xs);
    padding: 1px 6px;
    border-radius: var(--radius-xs);
    font-weight: 500;
    background: var(--bg-subtle);
    color: var(--text-secondary);
  }

  .row-status.ok {
    background: var(--status-ok-bg);
    color: var(--status-ok);
  }

  .row-status.warn {
    background: var(--status-warn-bg);
    color: var(--status-warn);
  }

  .row-status.danger {
    background: var(--status-danger-bg);
    color: var(--status-danger);
  }

  .row-desc {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.45;
  }

  .row-control {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
