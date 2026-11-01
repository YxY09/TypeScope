<script>
  export let columns = 5;
</script>

<section class="metric-strip" style="--cols: {columns}">
  <slot />
</section>

<style>
  .metric-strip {
    display: grid;
    grid-template-columns: repeat(var(--cols, 5), minmax(0, 1fr));
    background: var(--bg-surface);
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
  }

  .metric-strip :global(.metric-item + .metric-item) {
    border-left: 1px solid var(--border-subtle);
  }

  @media (max-width: 1120px) {
    .metric-strip {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .metric-strip :global(.metric-item) {
      border-bottom: 1px solid var(--border-subtle);
    }
  }
</style>
