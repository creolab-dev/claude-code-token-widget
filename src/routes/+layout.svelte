<script lang="ts">
  import '$lib/styles/globals.css';
  import { settingsState } from '$lib/state/settings-state.svelte';
  let { children } = $props();

  let fontVars = $derived(
    `--font-label: ${settingsState.fontSize}px; --font-value: ${settingsState.fontSize + 3}px; --font-small: ${settingsState.fontSize - 2}px; --font-tiny: ${settingsState.fontSize - 4}px`
  );
</script>

<div class="app {settingsState.theme}" style={fontVars}>
  {@render children()}
</div>

<style>
  .app {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }
</style>
