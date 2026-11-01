<script>
  export let text = "";
</script>

<span class="tooltip-wrap" title={text}>
  <slot />
</span>

<style>
  .tooltip-wrap {
    display: inline-flex;
    align-items: center;
    cursor: help;
  }
</style>
