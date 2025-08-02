<script lang="ts">
  // Example reactive state
  let statusText = 'Ready';
  let progress = 0.35; // 35%
  let details = 'No errors';
</script>

<div class="status-bar">
  <div class="pane left">{statusText}</div>
  <div class="pane center">
    <div class="progress-container" aria-label="progress">
      <div class="progress-bar" style="width: {Math.round(progress * 100)}%"></div>
    </div>
    <div class="progress-label">{Math.round(progress * 100)}%</div>
  </div>
  <div class="pane right">{details}</div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    padding: 0 8px;
    height: 26px;
    font-size: 12px;
    gap: 12px;
    background: var(--status-bg);
  }

  .pane {
    display: flex;
    align-items: center;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .left {
    flex: 1 1 150px;
  }
  .center {
    flex: 0 0 250px;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .right {
    flex: 1 1 120px;
    justify-content: flex-end;
  }

  .progress-container {
    flex: 1;
    background: var(--progress-bg);
    border-radius: 4px;
    height: 8px;
    overflow: hidden;
    position: relative;
  }

  .progress-bar {
    background: var(--progress-bar);
    height: 100%;
    transition: width 0.3s;
  }

  .progress-label {
    min-width: 32px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
