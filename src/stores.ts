import { writable } from "svelte/store";

export type FlatNode = {
    id: string;
    label: string;
    depth: number;
    expanded: boolean;
    version: number;
}

export const expandedPaths = writable<Set<string>>(new Set());
export const rows = writable<FlatNode[]>([]);
export const subscriberId = writable<string>(crypto.randomUUID());
export const viewportRange = writable<{ start: number; end: number }>({ start: 0, end: 50 });