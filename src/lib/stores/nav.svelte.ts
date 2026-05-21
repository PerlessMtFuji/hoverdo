// Main-window navigation state. Tiny on purpose: just which section is
// active in the sidebar. Persisted to localStorage so the user lands on
// the same view they last used.

export type Section = 'notes' | 'lists';

const STORAGE_KEY = 'hoverdo:nav:section';

function read(): Section {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === 'lists' ? 'lists' : 'notes';
  } catch {
    return 'notes';
  }
}

class NavStore {
  section = $state<Section>(read());

  set(next: Section): void {
    this.section = next;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // ignore - non-fatal
    }
  }
}

export const navStore = new NavStore();
