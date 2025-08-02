<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  
  import {
    activeScanId,
    statusText,
    details,
    percent,
    current,
    total,
    showProgress,
    resetScanState
  } from "$lib/scanStore";
  import { get } from "svelte/store";

  let unlistenStarted: (() => void) | undefined;
  let unlistenProgress: (() => void) | undefined;
  let unlistenFinished: (() => void) | undefined;
  let unlistenCancelled: (() => void) | undefined;
  let unlistenError: (() => void) | undefined;
  
  onMount(async () => {
    unlistenStarted = await listen('scan_started', (event: any) => {
      const payload = event.payload || {};
      const scanId = payload.scan_id as string | undefined;
      if (!scanId) return;

      activeScanId.set(scanId);
      showProgress.set(true);
      statusText.set('Scanning (Press ESC to cancel)...');
      percent.set(0);
      current.set(0);
      total.set(0);
      details.set('');
    });

    unlistenProgress = await listen('scan_progress', (event: any) => {
      const payload = event.payload || {};
      const scanId = payload.scan_id;
      if (scanId !== get(activeScanId)) return;

      percent.set(payload.percent ?? 0);
      current.set(payload.current ?? 0);
      total.set(payload.total ?? 0);
    });

    unlistenFinished = await listen('scan_finished', (event: any) => {
      const payload = event.payload || {};
      const scanId = payload.scan_id;
      if (scanId !== get(activeScanId)) return;

      statusText.set('Ready');
      setTimeout(() => {
        showProgress.set(false);
        percent.set(0);
        activeScanId.set(null);
      }, 300);
    });

    unlistenCancelled = await listen('scan_cancelled', (event: any) => {
      const payload = event.payload || {};
      const scanId = payload.scan_id;
      if (scanId !== get(activeScanId)) return;

      statusText.set('Cancelled');
      setTimeout(() => {
        // showProgress.set(false);
        // activeScanId.set(null);
        // percent.set(0);
        resetScanState();
      }, 300);
    });

    unlistenError = await listen('scan_error', (event: any) => {
      const payload = event.payload || {};
      const scanId = payload.scan_id;
      if (scanId !== get(activeScanId)) return;

      statusText.set('Error during scan');
      details.set(payload.error ?? '');
      setTimeout(() => {
        showProgress.set(false);
        activeScanId.set(null);
        percent.set(0);
      }, 1000);
    });
  });

  onDestroy(() => {
    unlistenStarted?.();
    unlistenProgress?.();
    unlistenFinished?.();
    unlistenCancelled?.();
    unlistenError?.();
  });  
</script>

<div class="status-bar">
  <div class="pane status-text">{$statusText}</div>
  
  {#if $showProgress}
    <div class="pane progress" aria-label="progress">
      <div class="progress-container" aria-label="progress">
        <div class="progress-bar" style="width: {$percent}%"></div>
      </div>
      <div class="progress-label">{$percent}% ({$current}/{$total})</div>      
    </div>
  {/if}

  <div class="pane details">{$details}</div>
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
