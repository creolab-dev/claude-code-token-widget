<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';

  let percentage = $derived(tokenState.usedPercentage);
  let color = $derived(tokenState.contextColor);
  let glowActive = $derived(percentage > 75);
</script>

<div class="context-bar">
  <span class="label">Context</span>
  <div class="bar-container">
    <div
      class="bar-fill"
      class:glow={glowActive}
      style="width: {percentage}%; background-color: {color}"
    ></div>
  </div>
  <span class="value" style="color: {color}">{percentage.toFixed(1)}%</span>
</div>

<style>
  .context-bar {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .label {
    font-size: var(--font-label);
    color: var(--text-secondary);
    width: 48px;
    flex-shrink: 0;
  }
  .bar-container {
    flex: 1;
    height: 8px;
    background: var(--bar-bg, rgba(128, 128, 128, 0.25));
    border-radius: 4px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease, background-color 0.3s ease;
  }
  .bar-fill.glow {
    box-shadow: 0 0 6px currentColor;
  }
  .value {
    font-size: var(--font-label);
    font-weight: 600;
    width: 36px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
