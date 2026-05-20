/**
 * Theme runtime: tracks light/dark resolution and source (user vs OS),
 * applies `data-theme` to the document root, and persists the user's choice.
 *
 * Persistence currently uses localStorage; once the Tauri SQL store is wired up
 * (post-scaffold commits) this becomes the single source of truth and syncs
 * across windows via the `theme:changed` Tauri event.
 */

type ThemeSource = 'system' | 'user';
type Resolved = 'light' | 'dark';

const STORAGE_KEY = 'hoverdo:theme';

interface ThemeState {
  resolved: Resolved;
  source: ThemeSource;
  /** What the user explicitly chose, if anything. */
  preference: Resolved | null;
}

export const theme = $state<ThemeState>({
  resolved: 'light',
  source: 'system',
  preference: null
});

function readStoredPreference(): Resolved | null {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === 'light' || v === 'dark' ? v : null;
  } catch {
    return null;
  }
}

function osScheme(): Resolved {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light';
}

function apply(resolved: Resolved, source: ThemeSource) {
  document.documentElement.dataset.theme = resolved;
  theme.resolved = resolved;
  theme.source = source;
}

export function initTheme(): void {
  const pref = readStoredPreference();
  if (pref) {
    theme.preference = pref;
    apply(pref, 'user');
  } else {
    apply(osScheme(), 'system');
  }

  // Follow OS only while no explicit preference is set
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  mq.addEventListener('change', (e) => {
    if (theme.preference) return;
    apply(e.matches ? 'dark' : 'light', 'system');
  });
}

export function setTheme(next: Resolved | null): void {
  theme.preference = next;
  try {
    if (next) localStorage.setItem(STORAGE_KEY, next);
    else localStorage.removeItem(STORAGE_KEY);
  } catch {
    // localStorage may be unavailable in webviews with strict policies; ignore.
  }
  apply(next ?? osScheme(), next ? 'user' : 'system');
}

export function toggleTheme(): void {
  setTheme(theme.resolved === 'dark' ? 'light' : 'dark');
}
