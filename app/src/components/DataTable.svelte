<script>
  export let columns = []; // [{ key, label, align: 'left'|'right', mono: boolean, width?: string }]
  export let rows = [];
  export let emptyText = "暂无记录";
  export let onRowClick = null;
  export let selectedKey = null;
  export let rowKey = "id";
</script>

{#if !rows || rows.length === 0}
  <div class="table-empty">{emptyText}</div>
{:else}
  <div class="table-wrap">
    <table class="data-table">
      <thead>
        <tr>
          {#each columns as col}
            <th
              style={col.width ? `width:${col.width}` : ""}
              class:right={col.align === "right"}
            >
              {col.label}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row, idx}
          <tr
            class:clickable={Boolean(onRowClick)}
            class:selected={selectedKey !== null && row[rowKey] === selectedKey}
            on:click={() => onRowClick && onRowClick(row, idx)}
          >
            {#each columns as col}
              <td
                class:right={col.align === "right"}
                class:mono={col.mono || col.align === "right"}
              >
                {#if col.render}
                  {@html col.render(row[col.key], row, idx)}
                {:else}
                  {row[col.key] ?? "—"}
                {/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .table-wrap {
    width: 100%;
    overflow-x: auto;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .data-table th {
    text-align: left;
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-muted);
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-default);
    white-space: nowrap;
  }

  .data-table td {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
    white-space: nowrap;
  }

  .data-table tr:last-child td {
    border-bottom: none;
  }

  .data-table tr.clickable {
    cursor: pointer;
  }

  .data-table tr.clickable:hover td {
    background: var(--bg-hover);
  }

  .data-table tr.selected td {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 500;
  }

  .right {
    text-align: right !important;
  }

  .table-empty {
    padding: 16px;
    text-align: center;
    font-size: var(--text-xs);
    color: var(--text-muted);
    background: var(--bg-subtle);
    border-radius: var(--radius-sm);
  }
</style>
