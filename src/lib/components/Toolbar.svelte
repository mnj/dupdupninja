<script lang="ts">
  import ThemeToggle from "./ThemeToggle.svelte";

  const toolbarActions = [
    { name: 'Refresh', icon: '⟳', onClick: () => console.log('Refresh') },
    { name: 'Add', icon: '+', onClick: () => console.log('Add') },
    { name: 'Delete', icon: '−', onClick: () => console.log('Delete') },
  ];
</script>

<div class="toolbar">
  {#each toolbarActions as act}
    <button aria-label={act.name} class="tool-btn" on:click={act.onClick} title={act.name}>
      <span class="icon">{act.icon}</span>
    </button>
  {/each}
  <div class="spacer"></div>
  <ThemeToggle />
</div>

<style>
  .toolbar {
    display: flex;
    gap: 6px;
    padding: 4px 8px;
    background: var(--toolbar-bg);
    border-bottom: 1px solid var(--border);
    height: var(--toolbar-height);
    align-items: center;
  }

  .tool-btn {
    background: var(--surface);
    color: var(--on-surface);
    border: 1px solid transparent;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;    
    display: flex;
    align-items: center;
    transition: background .2s ease, border-color .2s ease, box-shadow .2s ease;
    box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    overflow: hidden;
  }
 .tool-btn:hover {
    /* subtle overlay instead of hardcoded color so it adapts */
    background: 
      /* base surface plus overlay */
      linear-gradient(var(--btn-hover-overlay), var(--btn-hover-overlay)),
      var(--surface);
    border-color: var(--progress-bar); /* accent border on hover */
  }
  .tool-btn:active {
    background:
      linear-gradient(var(--btn-active-overlay), var(--btn-active-overlay)),
      var(--surface);
    border-color: var(--progress-bar);
    transform: translateY(1px);
  }
  .tool-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px var(--focus-ring);
  }
  .icon {
    display: inline-block;
    width: 1em;
    text-align: center;
  }
</style>
