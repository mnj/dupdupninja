<script lang="ts">
  import { onMount } from 'svelte';

  let container: HTMLDivElement;
  let leftPane: HTMLDivElement;
  let rightPane: HTMLDivElement;
  let separator: HTMLDivElement;

  // initial ratio 1:3
  let leftWidth = 0; // will set in onMount

  const minLeft = 100;
  const minRight = 200;

  function startDrag(e: PointerEvent) {
    e.preventDefault();
    separator.setPointerCapture(e.pointerId);
    const startX = e.clientX;
    const startLeftWidth = leftPane.getBoundingClientRect().width;

    function move(ev: PointerEvent) {
      const delta = ev.clientX - startX;
      let newLeft = startLeftWidth + delta;
      const total = container.clientWidth;
      const rightWidth = total - newLeft - separator.offsetWidth;
      if (newLeft < minLeft) newLeft = minLeft;
      if (rightWidth < minRight) newLeft = total - separator.offsetWidth - minRight;
      leftWidth = newLeft;
      leftPane.style.flex = `0 0 ${leftWidth}px`;
    }

    function up(ev: PointerEvent) {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      separator.releasePointerCapture(ev.pointerId);
    }

    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }

  onMount(() => {
    // set initial widths: container width * 1/(1+3) = 25%
    const total = container.clientWidth;
    leftWidth = Math.max(200, total * 0.25);
    leftPane.style.flex = `0 0 ${leftWidth}px`;
  });
</script>

<div class="split-root" bind:this={container}>
  <div class="pane left" bind:this={leftPane}>
    <div class="pane-title">File Sets</div>
    <div class="tree-wrapper">
      <!-- Replace with your actual tree component -->
      <div class="fake-tree">
        <div>📁 Root</div>
        <div style="margin-left:12px;">📁 Subfolder</div>
        <div style="margin-left:24px;">📄 File A</div>
        <div style="margin-left:24px;">📄 File B</div>
        <div style="margin-left:12px;">📁 Other</div>
      </div>
    </div>
  </div>
  <div class="separator" bind:this={separator} on:pointerdown={startDrag}></div>
  <div class="pane right" bind:this={rightPane}>
    <div class="pane-title">Detected Duplicates</div>
    <div class="table-wrapper">
      <!-- Placeholder for TanStack table with virtualization -->
      <div class="virtual-table">
        <!-- You'd integrate TanStack here; this is just a placeholder -->
        <div class="row header">Column A | Column B | Column C</div>
        {#each Array(200) as _, i}
          <div class="row">
            Row {i + 1}A | Row {i + 1}B | Row {i + 1}C
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .split-root {
    display: flex;
    flex: 1;
    overflow: hidden;
    background: var(--surface);
  }

  .pane {
    display: flex;
    flex-direction: column;
    overflow: hidden;

    min-width: 0;
  }

  .pane.left {
    /* Controlled by leftPane.style.flex set by the JS */
    flex: 0 0 auto; /* prevent flex-grow */
  }

  .pane.right {
    flex: 1 1 auto;
  }

  .pane-title {
    padding: 6px 10px;
    background: var(--muted-bg);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    color: var(--on-surface);
  }

  .tree-wrapper,
  .table-wrapper {
    flex: 1;
    overflow: auto;
    padding: 8px;
    min-width: 0; /* ensures contained scrolling works and prevents overflow */
  }

  .fake-tree {
    font-size: 13px;
    line-height: 1.5;
    background: var(--muted-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 10px;
    max-width: 100%;
    color: var(--on-surface);
  }

  .virtual-table {
    font-family: monospace;
    font-size: 12px;
    background: var(--muted-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
    display: flex;
    flex-direction: column;

    width: 100%;
    min-width: 0;
    color: var(--on-surface);
  }

  .row {
    padding: 6px 10px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    white-space: nowrap;
  }

  .row.header {
    background: rgba(255, 255, 255, 0.15);
    font-weight: 600;
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .separator {
    width: 6px;
    cursor: col-resize;
    background: transparent;
    position: relative;
    flex-shrink: 0;
  }
  .separator::after {
    content: '';
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--border);
    transform: translateX(-50%);
    border-radius: 1px;
  }
</style>
