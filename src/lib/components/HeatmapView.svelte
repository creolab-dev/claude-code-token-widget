<script lang="ts">
  import { heatmapState } from '$lib/state/heatmap-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import type { HeatmapEntry } from '$lib/types';

  interface Props {
    onBack: () => void;
  }
  let { onBack }: Props = $props();

  async function startDrag(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('button')) return;
    await getCurrentWebviewWindow().startDragging();
  }

  $effect(() => {
    heatmapState.load();
  });

  interface CellData {
    date: string;
    entry: HeatmapEntry | null;
    intensity: number;
  }

  let grid = $derived.by((): CellData[][] => {
    const entryMap = new Map<string, HeatmapEntry>();
    for (const e of heatmapState.entries) {
      entryMap.set(e.date, e);
    }

    const today = new Date();
    const weeks: CellData[][] = [];
    const todayDay = today.getDay();
    const totalDays = 26 * 7 + todayDay + 1;
    const startDate = new Date(today);
    startDate.setDate(startDate.getDate() - totalDays + 1);

    let currentWeek: CellData[] = [];
    for (let i = 0; i < totalDays; i++) {
      const d = new Date(startDate);
      d.setDate(d.getDate() + i);
      const dateStr = d.toISOString().slice(0, 10);
      const entry = entryMap.get(dateStr) ?? null;
      const cost = entry?.total_cost_usd ?? 0;

      currentWeek.push({
        date: dateStr,
        entry,
        intensity: heatmapState.getIntensity(cost),
      });

      if (currentWeek.length === 7) {
        weeks.push(currentWeek);
        currentWeek = [];
      }
    }
    if (currentWeek.length > 0) {
      weeks.push(currentWeek);
    }

    return weeks;
  });

  let months = $derived.by((): { label: string; col: number }[] => {
    const result: { label: string; col: number }[] = [];
    const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    let lastMonth = -1;

    for (let w = 0; w < grid.length; w++) {
      const firstDay = grid[w][0];
      if (!firstDay) continue;
      const month = new Date(firstDay.date).getMonth();
      if (month !== lastMonth) {
        result.push({ label: monthNames[month], col: w });
        lastMonth = month;
      }
    }
    return result;
  });

  let totalCost = $derived(
    heatmapState.entries.reduce((sum, e) => sum + e.total_cost_usd, 0)
  );

  let todayEntry = $derived.by(() => {
    const today = new Date().toISOString().slice(0, 10);
    return heatmapState.entries.find(e => e.date === today) ?? null;
  });

  let tooltipText = $state('');
  let tooltipX = $state(0);
  let tooltipY = $state(0);
  let showTooltip = $state(false);
  let tooltipAnchor = $state<'left' | 'right'>('left');

  function handleCellHover(cell: CellData, event: MouseEvent) {
    // 年を除いた日付表示 (M/D)
    const d = new Date(cell.date);
    const label = `${d.getMonth() + 1}/${d.getDate()}`;
    if (cell.entry) {
      tooltipText = `${label}: $${cell.entry.total_cost_usd.toFixed(4)} (${cell.entry.session_count} sessions)`;
    } else {
      tooltipText = `${label}: no data`;
    }
    // 右1/3ではマウス左側、左2/3ではマウス右側
    if (event.clientX > 200) {
      tooltipX = event.clientX - 6;
      tooltipAnchor = 'right';
    } else {
      tooltipX = event.clientX + 6;
      tooltipAnchor = 'left';
    }
    tooltipY = event.clientY - 6;
    showTooltip = true;
  }

  function hideTooltip() {
    showTooltip = false;
  }

  function intensityColor(level: number): string {
    const isDark = settingsState.theme === 'dark';
    if (isDark) {
      switch (level) {
        case 0: return 'rgba(255,255,255,0.06)';
        case 1: return '#0e4429';
        case 2: return '#006d32';
        case 3: return '#26a641';
        case 4: return '#39d353';
        default: return 'rgba(255,255,255,0.06)';
      }
    } else {
      switch (level) {
        case 0: return 'rgba(0,0,0,0.06)';
        case 1: return '#9be9a8';
        case 2: return '#40c463';
        case 3: return '#30a14e';
        case 4: return '#216e39';
        default: return 'rgba(0,0,0,0.06)';
      }
    }
  }

  // セルサイズ(px) — 曜日ラベル幅と月ラベル位置の計算に使用
  const cellSize = 7;
  const cellGap = 2;
  const dayLabelWidth = 24;
</script>

<div class="heatmap-panel">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="panel-header" onmousedown={startDrag} role="banner">
    <span class="panel-title">Usage Heatmap</span>
    <button class="header-done-btn" onclick={onBack}>Back</button>
  </div>

  <div class="panel-content">
    {#if heatmapState.loading}
      <div class="loading">Loading...</div>
    {:else}
      <div class="today-stats">
        <div class="today-label">Today</div>
        {#if todayEntry}
          <span class="today-value">${todayEntry.total_cost_usd.toFixed(4)}</span>
          <span class="today-detail">{todayEntry.session_count} sessions</span>
        {:else}
          <span class="today-value">no data</span>
        {/if}
      </div>

      <div class="summary">
        Total: ${totalCost.toFixed(2)} / {heatmapState.entries.length} days recorded
      </div>

      <!-- 月ラベル -->
      <div class="month-row" style="padding-left: {dayLabelWidth}px">
        {#each months as m, i}
          {@const nextCol = i + 1 < months.length ? months[i + 1].col : grid.length}
          {@const span = nextCol - m.col}
          <span class="month-label" style="width: {span * (cellSize + cellGap)}px">{m.label}</span>
        {/each}
      </div>

      <!-- グリッド + 曜日ラベル -->
      <div class="grid-wrapper">
        <div class="day-labels" style="width: {dayLabelWidth}px">
          <span class="day-label">&nbsp;</span>
          <span class="day-label">Mo</span>
          <span class="day-label">&nbsp;</span>
          <span class="day-label">We</span>
          <span class="day-label">&nbsp;</span>
          <span class="day-label">Fr</span>
          <span class="day-label">&nbsp;</span>
        </div>

        <div class="grid">
          {#each grid as week}
            <div class="week">
              {#each week as cell}
                <div
                  class="cell"
                  style="background-color: {intensityColor(cell.intensity)}; width: {cellSize}px; height: {cellSize}px"
                  onmouseenter={(e) => handleCellHover(cell, e)}
                  onmouseleave={hideTooltip}
                  role="presentation"
                ></div>
              {/each}
            </div>
          {/each}
        </div>
      </div>

      <div class="legend">
        <span class="legend-label">Less</span>
        {#each [0, 1, 2, 3, 4] as level}
          <div class="legend-cell" style="background-color: {intensityColor(level)}"></div>
        {/each}
        <span class="legend-label">More</span>
      </div>
    {/if}
  </div>
</div>

{#if showTooltip}
  <div
    class="tooltip"
    style="{tooltipAnchor === 'right' ? `right: ${300 - tooltipX}px` : `left: ${tooltipX}px`}; top: {tooltipY - 20}px"
  >
    {tooltipText}
  </div>
{/if}

<style>
  .heatmap-panel {
    width: 300px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    font-family: 'Segoe UI', system-ui, sans-serif;
    user-select: none;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    border-radius: var(--radius) var(--radius) 0 0;
    flex-shrink: 0;
    cursor: grab;
  }
  .panel-header:active {
    cursor: grabbing;
  }
  .panel-title {
    font-size: var(--font-small, 11px);
    font-weight: 600;
    color: var(--accent);
  }
  .header-done-btn {
    background: var(--accent);
    border: none;
    color: #fff;
    font-size: var(--font-small, 11px);
    font-weight: 600;
    cursor: pointer;
    padding: 3px 12px;
    border-radius: 4px;
  }
  .header-done-btn:hover {
    opacity: 0.9;
  }
  .panel-content {
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .loading {
    text-align: center;
    color: var(--text-secondary);
    font-size: var(--font-small, 11px);
    padding: 20px 0;
  }
  .today-stats {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 0;
    border-bottom: 1px solid var(--border);
  }
  .today-label {
    font-size: var(--font-small, 11px);
    color: var(--text-secondary);
    font-weight: 600;
  }
  .today-value {
    font-size: var(--font-label, 13px);
    color: var(--text-primary);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .today-detail {
    font-size: var(--font-small, 11px);
    color: var(--text-secondary);
    margin-left: auto;
  }
  .summary {
    font-size: var(--font-small, 11px);
    color: var(--text-secondary);
    text-align: center;
  }

  /* 月ラベル行 */
  .month-row {
    display: flex;
    height: 12px;
    overflow: hidden;
  }
  .month-label {
    font-size: 8px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: clip;
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* グリッド + 曜日ラベル */
  .grid-wrapper {
    display: flex;
    gap: 0;
  }
  .day-labels {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }
  .day-label {
    font-size: 8px;
    color: var(--text-secondary);
    height: 7px;
    line-height: 7px;
    overflow: hidden;
  }
  .grid {
    display: flex;
    gap: 2px;
    flex: 1;
    overflow: hidden;
  }
  .week {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cell {
    border-radius: 1px;
    flex-shrink: 0;
  }
  .legend {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
    padding-top: 2px;
  }
  .legend-label {
    font-size: var(--font-tiny, 9px);
    color: var(--text-secondary);
    padding: 0 2px;
  }
  .legend-cell {
    width: 8px;
    height: 8px;
    border-radius: 1px;
  }
  .tooltip {
    position: fixed;
    background: rgba(0, 0, 0, 0.88);
    color: #fff;
    font-size: var(--font-small, 11px);
    padding: 3px 6px;
    border-radius: 4px;
    pointer-events: none;
    z-index: 9999;
    white-space: nowrap;
    width: auto;
  }
</style>
