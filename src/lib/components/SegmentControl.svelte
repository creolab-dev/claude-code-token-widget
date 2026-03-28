<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }
  interface Props {
    options: Option[];
    selected: string;
    onchange: (value: string) => void;
  }
  let { options, selected, onchange }: Props = $props();
</script>

<div class="segment-control">
  {#each options as opt}
    <button
      class="segment"
      class:active={selected === opt.value}
      onclick={() => onchange(opt.value)}
    >
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .segment-control {
    display: flex;
    background: var(--bg-secondary);
    border-radius: 6px;
    padding: 2px;
    gap: 2px;
  }
  .segment {
    flex: 1;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 500;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }
  .segment.active {
    background: var(--accent);
    color: #fff;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }
</style>
