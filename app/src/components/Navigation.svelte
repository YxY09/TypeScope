<script>
  export let activeTab = "overview";
  export let onSelect = () => {};

  const items = [
    { id: "overview", label: "总览" },
    { id: "analytics", label: "分析" },
    { id: "text", label: "文字" },
    { id: "settings", label: "设置" }
  ];
</script>

<nav class="primary-nav" aria-label="主导航">
  {#each items as item}
    <button
      type="button"
      class="nav-item"
      class:active={activeTab === item.id}
      on:click={() => onSelect(item.id)}
    >
      {item.label}
    </button>
  {/each}
</nav>

<style>
  .primary-nav {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    background: var(--bg-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .nav-item {
    height: 26px;
    padding: 0 14px;
    font-size: var(--text-base);
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    transition: color 0.12s ease, background-color 0.12s ease;
  }

  .nav-item:hover:not(.active) {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .nav-item.active {
    color: var(--accent-fg);
    font-weight: 600;
    background: var(--accent);
  }
</style>
