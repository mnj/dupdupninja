<script lang="ts">
  import { theme } from '../theme'
  import { derived } from 'svelte/store';
  import { onMount } from 'svelte';

  // effective resolved theme (light/dark) for icon/display
  const effective = derived(theme, ($theme) => {
    if ($theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return $theme;
  });

  function cycle() {
    theme.update((t) => {
      if (t === 'system') return 'dark';
      if (t === 'dark') return 'light';
      return 'system';
    });
  }
</script>

<button class="theme-toggle" on:click={cycle} aria-label="Toggle theme">
  {#if $effective === 'dark'}
    🌙
  {:else}
    ☀️
  {/if}
  <small class="mode-label">{$theme === 'system' ? 'Auto' : $theme}</small>
</button>

<style>
  .theme-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    border: 1px solid transparent;
    border-radius: 4px;
    background: var(--surface);
    cursor: pointer;
    font-size: 12px;
  }

  .theme-toggle:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .mode-label {
    font-size: 9px;
    opacity: 0.75;
    margin-left: 2px;
  }
</style>
