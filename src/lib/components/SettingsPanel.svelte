<script lang="ts">
  import { settingsState } from '$lib/state/settings-state.svelte';
  import ToggleSwitch from './ToggleSwitch.svelte';
  import SegmentControl from './SegmentControl.svelte';
  import { setAlwaysOnTopCmd } from '$lib/tauri/commands';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  async function startDrag(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('button')) return;
    await getCurrentWebviewWindow().startDragging();
  }

  let { closeSettings }: { closeSettings: () => void } = $props();

  function handleOpacityChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.opacity = parseFloat(target.value);
    settingsState.save();
  }

  function handleFontSizeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.fontSize = parseInt(target.value, 10);
    settingsState.save();
  }

  function toggleDisplay(key: string, value: boolean) {
    settingsState.displayItems[key] = value;
    settingsState.save();
  }

  function setTheme(value: string) {
    settingsState.theme = value as 'dark' | 'light';
    settingsState.save();
  }

  async function setAlwaysOnTop(value: boolean) {
    settingsState.alwaysOnTop = value;
    try {
      await setAlwaysOnTopCmd(value);
    } catch (e) {
      console.error('Failed to set always on top:', e);
    }
    settingsState.save();
  }

  function setAlertEnabled(value: boolean) {
    settingsState.alertSettings.enabled = value;
    settingsState.save();
  }

  function toggleThreshold(threshold: number) {
    const idx = settingsState.alertSettings.thresholds.indexOf(threshold);
    if (idx >= 0) {
      settingsState.alertSettings.thresholds = settingsState.alertSettings.thresholds.filter((t) => t !== threshold);
    } else {
      settingsState.alertSettings.thresholds = [...settingsState.alertSettings.thresholds, threshold].sort((a, b) => a - b);
    }
    settingsState.save();
  }

  function setScheduleEnabled(value: boolean) {
    settingsState.scheduleSettings.enabled = value;
    settingsState.save();
  }

  function handleHourChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.scheduleSettings.wakeHour = parseInt(target.value, 10);
    settingsState.save();
  }

  function handleMinuteChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.scheduleSettings.wakeMinute = parseInt(target.value, 10);
    settingsState.save();
  }

  function handleCommandChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.scheduleSettings.command = target.value;
    settingsState.save();
  }

  function handlePromptChange(e: Event) {
    const target = e.target as HTMLInputElement;
    settingsState.scheduleSettings.prompt = target.value;
    settingsState.save();
  }

  const thresholdOptions = [50, 75, 90];
</script>

<div class="panel">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="panel-header" onmousedown={startDrag} role="banner">
    <span class="panel-title">Settings</span>
    <button class="header-done-btn" onclick={closeSettings}>Done</button>
  </div>

  <div class="panel-content">
    <div class="section">
      <div class="section-title">THEME</div>
      <SegmentControl
        options={[{ value: 'dark', label: 'Dark' }, { value: 'light', label: 'Light' }]}
        selected={settingsState.theme}
        onchange={setTheme}
      />
    </div>

    <div class="section">
      <div class="section-title">DISPLAY</div>
      <ToggleSwitch label="Rate Limits" checked={settingsState.displayItems.rateLimits} onchange={(v) => toggleDisplay('rateLimits', v)} />
      <ToggleSwitch label="Context Window" checked={settingsState.displayItems.contextWindow} onchange={(v) => toggleDisplay('contextWindow', v)} />
      <ToggleSwitch label="Token Stats" checked={settingsState.displayItems.tokenStats} onchange={(v) => toggleDisplay('tokenStats', v)} />
      <ToggleSwitch label="Cost / Time" checked={settingsState.displayItems.cost} onchange={(v) => toggleDisplay('cost', v)} />
    </div>

    <div class="section">
      <div class="section-title">OPACITY: {Math.round(settingsState.opacity * 100)}%</div>
      <input
        type="range"
        min="0.3"
        max="1"
        step="0.05"
        value={settingsState.opacity}
        oninput={handleOpacityChange}
        class="slider"
      />
    </div>

    <div class="section">
      <div class="section-title">FONT SIZE: {settingsState.fontSize}px</div>
      <input
        type="range"
        min="10"
        max="18"
        step="1"
        value={settingsState.fontSize}
        oninput={handleFontSizeChange}
        class="slider"
      />
    </div>

    <div class="section">
      <ToggleSwitch label="Always on Top" checked={settingsState.alwaysOnTop} onchange={setAlwaysOnTop} />
    </div>

    <div class="section">
      <div class="section-title">ALERTS</div>
      <ToggleSwitch label="Notifications" checked={settingsState.alertSettings.enabled} onchange={setAlertEnabled} />
      {#if settingsState.alertSettings.enabled}
        <div class="threshold-group">
          {#each thresholdOptions as t}
            <label class="threshold-item">
              <input
                type="checkbox"
                checked={settingsState.alertSettings.thresholds.includes(t)}
                onchange={() => toggleThreshold(t)}
              />
              <span>{t}%</span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    <div class="section">
      <div class="section-title">WAKE-UP SCHEDULE</div>
      <ToggleSwitch label="Enabled" checked={settingsState.scheduleSettings.enabled} onchange={setScheduleEnabled} />
      {#if settingsState.scheduleSettings.enabled}
        <div class="schedule-time">
          <label class="time-input">
            <span class="time-label">Hour</span>
            <input
              type="number"
              min="0"
              max="23"
              value={settingsState.scheduleSettings.wakeHour}
              onchange={handleHourChange}
              class="time-field"
            />
          </label>
          <span class="time-colon">:</span>
          <label class="time-input">
            <span class="time-label">Min</span>
            <input
              type="number"
              min="0"
              max="59"
              step="5"
              value={settingsState.scheduleSettings.wakeMinute}
              onchange={handleMinuteChange}
              class="time-field"
            />
          </label>
        </div>
        <div class="schedule-hint">
          Claude CLI will start at {String(settingsState.scheduleSettings.wakeHour).padStart(2, '0')}:{String(settingsState.scheduleSettings.wakeMinute).padStart(2, '0')} to begin the rate limit window early.
        </div>
        <div class="schedule-fields">
          <label class="field-row">
            <span class="field-label">Command</span>
            <input
              type="text"
              value={settingsState.scheduleSettings.command}
              onchange={handleCommandChange}
              class="text-field"
              placeholder="claude"
            />
          </label>
          <label class="field-row">
            <span class="field-label">Prompt</span>
            <input
              type="text"
              value={settingsState.scheduleSettings.prompt}
              onchange={handlePromptChange}
              class="text-field"
              placeholder="hello"
            />
          </label>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .panel {
    width: 300px;
    height: auto;
    -webkit-app-region: no-drag;
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
  .panel-content {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .section {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
  }
  .section:last-child {
    border-bottom: none;
  }
  .section-title {
    font-size: var(--font-tiny, 9px);
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.5px;
    margin-bottom: 6px;
  }
  .slider {
    width: 100%;
    accent-color: var(--accent);
    cursor: pointer;
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
  .threshold-group {
    display: flex;
    gap: 12px;
    padding: 4px 0;
  }
  .threshold-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: var(--font-small, 11px);
    color: var(--text-primary);
    cursor: pointer;
  }
  .threshold-item input[type="checkbox"] {
    accent-color: var(--accent);
    cursor: pointer;
  }
  .schedule-time {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 4px 0;
  }
  .time-input {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .time-label {
    font-size: var(--font-tiny, 9px);
    color: var(--text-secondary);
  }
  .time-field {
    width: 48px;
    padding: 3px 6px;
    font-size: var(--font-label, 12px);
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    text-align: center;
  }
  .time-colon {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-secondary);
    padding-bottom: 3px;
  }
  .schedule-hint {
    font-size: var(--font-tiny, 9px);
    color: var(--text-secondary);
    line-height: 1.3;
    padding: 2px 0;
  }
  .schedule-fields {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 0;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: default;
  }
  .field-label {
    font-size: var(--font-tiny, 9px);
    color: var(--text-secondary);
    width: 56px;
    flex-shrink: 0;
  }
  .text-field {
    flex: 1;
    padding: 3px 6px;
    font-size: var(--font-small, 11px);
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
</style>
