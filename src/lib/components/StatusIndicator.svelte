<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';
  import type { ConnectionStatus } from '$lib/types';

  let status = $derived(tokenState.status);

  let color = $derived.by(() => {
    const s = status;
    if (s === 'loading') return '#eab308';
    if (s === 'connected') return '#22c55e';
    if (s === 'error') return '#ef4444';
    return '#666666';
  });

  let pulsing = $derived(status === 'loading' || status === 'error');
</script>

<div class="status-indicator" title={status}>
  <div class="dot" class:pulse={pulsing} style="background-color: {color}"></div>
</div>

<style>
  .status-indicator {
    position: absolute;
    top: 7px;
    left: 8px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    transition: background-color 0.3s ease;
  }
  .dot.pulse {
    animation: pulse 1.5s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
</style>
