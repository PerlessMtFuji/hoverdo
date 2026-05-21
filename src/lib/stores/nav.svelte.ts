// Main-window navigation state. Tracks which section is active in the
// sidebar. Persisted to localStorage so the user lands on the same view
// they last used.
//
// 'home' is the new widget-hub landing surface; 'notes' and 'lists' are
// the library views.

export type Section = 'home' | 'notes' | 'lists';

const STORAGE_KEY = 'hoverdo:nav:section';

function read(): Section {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === 'notes' || v === 'lists' || v === 'home') return v;
  } catch {
    // ignore
  }
  return 'home';
}

class NavStore {
  section = $state<Section>(read());

  set(next: Section): void {
    this.section = next;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // ignore
    }
  }
}

export const navStore = new NavStore();
