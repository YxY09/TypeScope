<script>
  export let title = "暂无数据";
  export let description = "";
  export let compact = false;
</script>

<div class="empty-state" class:compact>
  <div class="empty-title">{title}</div>
  {#if description}
    <div class="empty-desc">{description}</div>
  {/if}
  <slot />
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 28px 16px;
    text-align: center;
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-md);
    background: var(--bg-subtle);
  }

  .empty-state.compact {
    padding: 16px 12px;
  }

  .empty-title {
    font-size: var(--text-base);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .empty-desc {
    font-size: var(--text-xs);
    color: var(--text-muted);
    max-width: 420px;
  }
</style>
