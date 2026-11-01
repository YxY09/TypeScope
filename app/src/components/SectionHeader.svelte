<script>
  export let title = "";
  export let subtitle = "";
  export let meta = "";
</script>

<header class="section-header">
  <div class="header-left">
    <h2 class="section-title">{title}</h2>
    {#if subtitle}
      <span class="section-subtitle">{subtitle}</span>
    {/if}
  </div>
  <div class="header-right">
    {#if meta}
      <span class="section-meta mono">{meta}</span>
    {/if}
    <slot />
  </div>
</header>

<style>
  .section-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--sp-3);
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-width: 0;
  }

  .section-title {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .section-subtitle {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .section-meta {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
