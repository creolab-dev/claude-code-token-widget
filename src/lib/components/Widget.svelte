<script lang="ts">
  import DragHandle from './DragHandle.svelte';
  import RateLimitBar from './RateLimitBar.svelte';
  import ContextBar from './ContextBar.svelte';
  import TokenStats from './TokenStats.svelte';
  import CostDisplay from './CostDisplay.svelte';
  import StatusIndicator from './StatusIndicator.svelte';
  import SettingsPanel from './SettingsPanel.svelte';
  import HeatmapView from './HeatmapView.svelte';
  import { tokenState } from '$lib/state/token-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { resizeWindow } from '$lib/tauri/commands';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  type ViewMode = 'widget' | 'settings' | 'heatmap';
  let viewMode = $state<ViewMode>('widget');
  let initialResizeDone = false;
  let widgetEl: HTMLDivElement | undefined = $state();
  let settingsEl: HTMLDivElement | undefined = $state();
  let heatmapEl: HTMLDivElement | undefined = $state();

  function updateWindowSize() {
    // 二重RAF + 少し遅延で初回レンダリング完了を確実に待つ
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        let h = 0;
        if (viewMode === 'settings' && settingsEl) {
          h = settingsEl.scrollHeight + 2;
        } else if (viewMode === 'heatmap' && heatmapEl) {
          h = heatmapEl.scrollHeight + 2;
        } else if (viewMode === 'widget' && widgetEl) {
          h = widgetEl.scrollHeight + 2;
        }
        if (h > 0) {
          resizeWindow(300, h).catch(() => {});
        }
      });
    });
  }

  // 起動直後にも再計測（設定読み込み後のレイアウト確定用）
  $effect(() => {
    if (tokenState.data && !initialResizeDone) {
      initialResizeDone = true;
      setTimeout(updateWindowSize, 200);
    }
  });

  $effect(() => {
    const _ = [
      settingsState.displayItems.rateLimits,
      settingsState.displayItems.contextWindow,
      settingsState.displayItems.tokenStats,
      settingsState.displayItems.cost,
      settingsState.fontSize,
      settingsState.alertSettings.enabled,
      settingsState.scheduleSettings.enabled,
      viewMode,
    ];
    updateWindowSize();
  });

  function openSettings() {
    viewMode = 'settings';
  }

  function openHeatmap() {
    viewMode = 'heatmap';
  }

  function backToWidget() {
    viewMode = 'widget';
  }

  async function closeApp() {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.close();
  }
</script>

<div class="widget-view" class:hidden={viewMode !== 'widget'}>
  <div class="widget" style="opacity: {settingsState.opacity}" bind:this={widgetEl}>
    <DragHandle onSettingsClick={openSettings} onHeatmapClick={openHeatmap} onCloseClick={closeApp} />
    <div class="widget-body">
      {#if settingsState.displayItems.rateLimits}
        <div class="rate-limits">
          <RateLimitBar
            label="5-Hour Limit"
            percentage={tokenState.fiveHourUsed}
            color={tokenState.fiveHourColor}
            resetTime={tokenState.fiveHourResetsAt}
          />
          <RateLimitBar
            label="7-Day Limit"
            percentage={tokenState.sevenDayUsed}
            color={tokenState.sevenDayColor}
            resetTime={tokenState.sevenDayResetsAt}
          />
        </div>
      {/if}
      {#if settingsState.displayItems.contextWindow}
        {#if settingsState.displayItems.rateLimits}
          <div class="divider"></div>
        {/if}
        <ContextBar />
      {/if}
      {#if settingsState.displayItems.tokenStats}
        <TokenStats />
      {/if}
      {#if settingsState.displayItems.cost}
        <CostDisplay />
      {/if}
    </div>
    <StatusIndicator />
  </div>
</div>

<div class="settings-view" class:hidden={viewMode !== 'settings'} bind:this={settingsEl}>
  <SettingsPanel closeSettings={backToWidget} />
</div>

<div class="heatmap-view" class:hidden={viewMode !== 'heatmap'} bind:this={heatmapEl}>
  <HeatmapView onBack={backToWidget} />
</div>

<style>
  .hidden {
    display: none !important;
  }
  .widget {
    width: 300px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    font-family: 'Segoe UI', system-ui, sans-serif;
    user-select: none;
    position: relative;
  }
  .widget-body {
    padding: 6px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .rate-limits {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .divider {
    height: 1px;
    background: var(--border);
    margin: 2px 0;
  }
</style>
