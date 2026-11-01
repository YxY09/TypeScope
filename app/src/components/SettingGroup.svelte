<script>
  export let title = "";
  export let description = "";
</script>

<section class="setting-group">
  <header class="group-header">
    <h3 class="group-title">{title}</h3>
    {#if description}
      <p class="group-desc">{description}</p>
    {/if}
  </header>
  <div class="group-body">
    <slot />
  </div>
</section>

<style>
  .setting-group {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: var(--sp-6);
    padding: var(--sp-4) 0;
    border-bottom: 1px solid var(--border-subtle);
    align-items: start;
  }

  .setting-group:last-child {
    border-bottom: none;
  }

  .group-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .group-title {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
  }

  .group-desc {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.45;
  }

  .group-body {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    divide-y: 1px solid var(--border-subtle);
  }

  @media (max-width: 960px) {
    .setting-group {
      grid-template-columns: 1fr;
      gap: var(--sp-3);
    }
  }
</style>
