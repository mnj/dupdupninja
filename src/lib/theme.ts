import type { MediaQuery } from "svelte/reactivity";
import { writable } from "svelte/store";

const STORAGE_KEY = 'ui-theme'; // "light" | "dark" | "system"

type Theme = 'light' | 'dark' | 'system';

function getPreferredSystemTheme(): 'light' | 'dark' {
    if (typeof window === 'undefined') return 'light';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveEffectiveTheme(theme: Theme): 'light' | 'dark' {
    return theme === 'system' ? getPreferredSystemTheme() : theme;
}

// Initialize from localStorage or default to 'system'
let initialTheme: Theme = 'system';
try{
    const stored = localStorage.getItem(STORAGE_KEY) as Theme | null;
    if (stored && (stored === 'light' || stored === 'dark' || stored === 'system')) {
        initialTheme = stored;
    }
} catch (error) {
    console.error('Error reading theme from localStorage:', error);
}

export const theme = writable<Theme>(initialTheme);

function apply(effectiveTheme: 'light' | 'dark') {
    document.documentElement.setAttribute('data-theme', effectiveTheme);
}

let mediaQuery : MediaQueryList | null = null;
if (typeof window !== 'undefined') {
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  // react to system changes when in 'system' mode
  mediaQuery.addEventListener?.('change', () => {
    theme.update((t) => {
      const eff = resolveEffectiveTheme(t);
      apply(eff);
      return t; // no change
    });
  });
}

// Subscribe and react
theme.subscribe((t) => {
  const effective = resolveEffectiveTheme(t);
  apply(effective);
  try {
    localStorage.setItem(STORAGE_KEY, t);
  } catch {}
});
