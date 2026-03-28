<script lang="ts">
  interface Props {
    checked: boolean;
    onchange: (checked: boolean) => void;
    label: string;
  }
  let { checked, onchange, label }: Props = $props();
</script>

<label class="toggle-switch">
  <span class="toggle-label">{label}</span>
  <button
    class="track"
    class:on={checked}
    onclick={() => onchange(!checked)}
    role="switch"
    aria-checked={checked}
  >
    <span class="thumb"></span>
  </button>
</label>

<style>
  .toggle-switch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
    cursor: pointer;
  }
  .toggle-label {
    font-size: var(--font-label, 12px);
    color: var(--text-primary);
  }
  .track {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: #555;
    border: none;
    padding: 2px;
    cursor: pointer;
    transition: background 0.2s ease;
    position: relative;
    flex-shrink: 0;
  }
  .track.on {
    background: #34c759;
  }
  .thumb {
    display: block;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s ease;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }
  .track.on .thumb {
    transform: translateX(16px);
  }
</style>
