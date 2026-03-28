<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { formatCostUsd, formatCostJpy, formatDuration } from '$lib/utils/format';

  let costFormatted = $derived(
    settingsState.currency === 'jpy'
      ? formatCostJpy(tokenState.costUsd, settingsState.jpyRate)
      : formatCostUsd(tokenState.costUsd)
  );

  let durationFormatted = $derived(formatDuration(tokenState.durationMs));
</script>

<div class="cost-display">
  <span class="cost">{costFormatted}</span>
  <span class="duration">{durationFormatted}</span>
</div>

<style>
  .cost-display {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-top: 4px;
    border-top: 1px solid var(--border);
    font-size: var(--font-label);
  }
  .cost {
    color: var(--text-primary);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .duration {
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    font-size: var(--font-small);
  }
</style>
