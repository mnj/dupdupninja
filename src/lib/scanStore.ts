// src/lib/scanStore.ts
import { writable } from 'svelte/store';

export type ScanPhase = 'idle' | 'discover' | 'processing' | 'finished' | 'cancelled' | 'error';

export const activeScanId = writable<string | null>(null);
export const statusText = writable('Ready');
export const details = writable('');
export const percent = writable(0);
export const current = writable(0);
export const total = writable(0);
export const showProgress = writable(false);
export const phase = writable<ScanPhase>('idle');
export const discovered = writable(0);

export function resetScanState() {
  activeScanId.set(null);
  statusText.set('Ready');
  details.set('');
  percent.set(0);
  current.set(0);
  total.set(0);
  showProgress.set(false);
  phase.set('idle');
  discovered.set(0);
}
