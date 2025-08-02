<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let statusText = $state<string>('Ready');
  let details = $state<string>('');

  // Receive updates from the backend
  let percent = $state<number>(0);
  let current = $state<number>(0);
  let total = $state<number>(0);
  let show_progress = $state<boolean>(false);

  let unlistenStarted: (() => void) | undefined;
  let unlistenProgress: (() => void) | undefined;
  let unlistenFinished: (() => void) | undefined;
  
  onMount(async () => {
    unlistenStarted = await listen('scan_started', () => {
      show_progress = true;
      statusText = 'Scanning...';
      percent = 0;
      current = 0;
      total = 0;
    });

    unlistenProgress = await listen('scan_progress', (event: any) => {
      if (event.payload) {
        percent = event.payload.percent ?? 0;
        current = event.payload.current ?? 0;
        total = event.payload.total ?? 0;
      }
    });

    unlistenFinished = await listen('scan_finished', () => {
      statusText = 'Ready';
      setTimeout(() => {
        show_progress = false;
        percent = 0;
      }, 300); // small delay so user sees completion
    });
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
    if (unlistenStarted) unlistenStarted();
    if (unlistenFinished) unlistenFinished();
  });  
</script>

<div class="status-bar">
  <div class="pane status-text">{statusText}</div>
  
  {#if show_progress}
  <div class="pane progress" aria-label="progress">
      <div class="progress-container" aria-label="progress">
        <div class="progress-bar" style="width: {percent}%"></div>
      </div>
      <div class="progress-label">{percent}% ({current}/{total})</div>
    </div>
  {/if}

  <div class="pane details">{details}</div>
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
    min-width: 0;
  }

  .pane {
    display: flex;
    align-items: center;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    min-width: 0;
    flex: none;
  }

  .status-text {
    max-width: 200px;
    min-width: 0;
    flex: 0 1 auto;
  }
  .progress {
    flex: 1 1 220px;
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
   }
  .details {
    flex: 0 1 auto;    
    min-width: 0;
  }

  .progress-container {
    flex: 1 1 auto;
    background: var(--progress-bg);
    border-radius: 4px;
    height: 8px;
    overflow: hidden;
    position: relative;
    min-width: 0;
  }

  .progress-bar {
    background: var(--progress-bar);
    height: 100%;
    transition: width 0.3s;
    width: 0%;
  }

  .progress-label {
    min-width: 32px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    flex: none;
  }
</style>
