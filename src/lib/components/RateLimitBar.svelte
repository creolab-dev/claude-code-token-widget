<script lang="ts">
  interface Props {
    label: string;
    percentage: number;
    color: string;
    resetTime: Date | null;
  }
  let { label, percentage, color, resetTime }: Props = $props();

  let tick = $state(0);
  $effect(() => {
    const interval = setInterval(() => { tick++; }, 60000);
    return () => clearInterval(interval);
  });

  let resetLabel = $derived.by(() => {
    const _ = tick; // force re-evaluation every 60s
    if (!resetTime) return '';
    const now = new Date();
    const diff = resetTime.getTime() - now.getTime();
    if (diff <= 0) return 'resetting...';
    const hours = Math.floor(diff / 3600000);
    const mins = Math.floor((diff % 3600000) / 60000);
    const hh = resetTime.getHours().toString().padStart(2, '0');
    const mm = resetTime.getMinutes().toString().padStart(2, '0');
    const remaining = hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
    if (hours >= 24) {
      const month = resetTime.getMonth() + 1;
      const day = resetTime.getDate();
      return `${month}/${day} ${hh}:${mm} (${remaining})`;
    }
    return `${hh}:${mm} (${remaining})`;
  });
</script>

<div class="rate-limit-bar">
  <div class="header">
    <span class="label">{label}</span>
    <span class="value" style="color: {color}">{percentage.toFixed(1)}%</span>
  </div>
  <div class="bar-container">
    <div
      class="bar-fill"
      class:glow={percentage > 75}
      style="width: {percentage}%; background-color: {color}"
    ></div>
  </div>
  {#if resetTime}
    <span class="reset">resets at {resetLabel}</span>
  {/if}
</div>

<style>
  .rate-limit-bar {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .label {
    font-size: var(--font-label);
    color: var(--text-secondary);
  }
  .value {
    font-size: var(--font-value);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .bar-container {
    height: 6px;
    background: var(--bar-bg, rgba(128, 128, 128, 0.25));
    border-radius: 3px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.3s ease, background-color 0.3s ease;
    min-width: 2px;
  }
  .bar-fill.glow {
    box-shadow: 0 0 6px currentColor;
  }
  .reset {
    font-size: var(--font-small);
    color: var(--text-secondary);
    text-align: right;
  }
</style>
