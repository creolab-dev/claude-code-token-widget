<script lang="ts">
  import StatRow from './StatRow.svelte';
  import { tokenState } from '$lib/state/token-state.svelte';
  import { formatTokens } from '$lib/utils/format';

  let inputFormatted = $derived(formatTokens(tokenState.totalInputTokens));
  let outputFormatted = $derived(formatTokens(tokenState.totalOutputTokens));
  let cacheFormatted = $derived(formatTokens(tokenState.cacheReadTokens));
</script>

<div class="token-stats">
  <StatRow label="Input" value={inputFormatted} unit="tokens" />
  <StatRow label="Output" value={outputFormatted} unit="tokens" />
  <StatRow label="Cache" value={cacheFormatted} unit="tokens" />
</div>

<style>
  .token-stats {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
