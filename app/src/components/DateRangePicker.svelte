<script>
  import { fmtDay } from "../lib/format.js";

  export let preset = "today";
  export let customStartInput = "";
  export let customEndInput = "";
  export let currentRange = { start: 0, end: 0 };
  export let onChangePreset = () => {};
  export let onChangeCustom = () => {};

  const presets = [
    { id: "today", label: "今天" },
    { id: "7d", label: "7 天" },
    { id: "30d", label: "30 天" },
    { id: "90d", label: "90 天" },
    { id: "custom", label: "自定义" }
  ];
</script>

<div class="range-picker">
  <div class="preset-group" role="group" aria-label="时间范围筛选">
    {#each presets as item}
      <button
        type="button"
        class="preset-btn"
        class:active={preset === item.id}
        on:click={() => onChangePreset(item.id)}
      >
        {item.label}
      </button>
    {/each}
  </div>

  {#if preset === "custom"}
    <div class="custom-inputs">
      <input
        type="date"
        class="date-input mono"
        bind:value={customStartInput}
        on:change={() => onChangeCustom(customStartInput, customEndInput)}
      />
      <span class="sep">—</span>
      <input
        type="date"
        class="date-input mono"
        bind:value={customEndInput}
        on:change={() => onChangeCustom(customStartInput, customEndInput)}
      />
    </div>
  {:else if currentRange.start && currentRange.end}
    <span class="range-caption mono">
      {#if currentRange.start === currentRange.end}
        {fmtDay(currentRange.start)}
      {:else}
        {fmtDay(currentRange.start)} 至 {fmtDay(currentRange.end)}
      {/if}
    </span>
  {/if}
</div>

<style>
  .range-picker {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .preset-group {
    display: inline-flex;
    align-items: center;
    background: var(--bg-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 2px;
    gap: 1px;
  }

  .preset-btn {
    height: 24px;
    padding: 0 9px;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-xs);
    transition: color 0.12s ease, background-color 0.12s ease;
  }

  .preset-btn:hover:not(.active) {
    color: var(--text-primary);
  }

  .preset-btn.active {
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
  }

  .custom-inputs {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .date-input {
    height: 26px;
    padding: 0 6px;
    font-size: var(--text-xs);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
  }

  .sep {
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .range-caption {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  @media (max-width: 1120px) {
    .range-caption {
      display: none;
    }
  }
</style>
