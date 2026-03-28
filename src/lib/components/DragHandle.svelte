<script lang="ts">
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  interface Props {
    onSettingsClick: () => void;
    onHeatmapClick: () => void;
    onCloseClick: () => void;
  }
  let { onSettingsClick, onHeatmapClick, onCloseClick }: Props = $props();

  async function startDrag(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.btn')) return;
    const window = getCurrentWebviewWindow();
    await window.startDragging();
  }
</script>

<div class="drag-handle" onmousedown={startDrag} role="banner">
  <span class="title">Token Widget</span>
  <div class="actions">
    <button class="btn heatmap" onclick={onHeatmapClick} title="Heatmap">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
        <rect x="0" y="8" width="3" height="4" rx="0.5" />
        <rect x="4.5" y="4" width="3" height="8" rx="0.5" />
        <rect x="9" y="0" width="3" height="12" rx="0.5" />
      </svg>
    </button>
    <button class="btn settings" onclick={onSettingsClick} title="Settings">&#9881;</button>
    <button class="btn close" onclick={onCloseClick} title="Close">&times;</button>
  </div>
</div>

<style>
  .drag-handle {
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 6px 0 18px;
    cursor: grab;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    border-radius: var(--radius) var(--radius) 0 0;
    flex-shrink: 0;
  }
  .drag-handle:active {
    cursor: grabbing;
  }
  .title {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.3px;
  }
  .actions {
    display: flex;
    gap: 2px;
  }
  .btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn:hover {
    color: var(--text-primary);
    background: rgba(128, 128, 128, 0.2);
  }
  .btn.close:hover {
    color: #fff;
    background: #ef4444;
  }
</style>
