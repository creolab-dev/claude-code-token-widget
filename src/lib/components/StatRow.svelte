<script lang="ts">
  interface Props {
    label: string;
    value: string;
    unit: string;
  }
  let { label, value, unit }: Props = $props();
</script>

<div class="stat-row">
  <span class="label">{label}</span>
  <span class="value">{value}</span>
  <span class="unit">{unit}</span>
</div>

<style>
  .stat-row {
    display: flex;
    align-items: baseline;
    gap: 4px;
    font-size: var(--font-label);
    line-height: 1.6;
  }
  .label {
    color: var(--text-secondary);
    width: 48px;
    flex-shrink: 0;
  }
  .value {
    color: var(--text-primary);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    flex: 1;
    text-align: right;
  }
  .unit {
    color: var(--text-secondary);
    font-size: var(--font-small);
    width: 40px;
    flex-shrink: 0;
  }
</style>
