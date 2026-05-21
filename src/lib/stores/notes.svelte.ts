// Notes store - single source of truth for the main window's note list.
// Reactive via Svelte 5 runes; consumed by NoteList / NoteEditor / Sidebar.
//
// We keep the store in-memory and re-fetch on `notes:changed` events
// emitted by the backend. Widget windows will subscribe to the same event,
// so any edit propagates without manual refresh.

import * as api from '$lib/ipc/commands';
import { on } from '$lib/ipc/events';
import type { Note, NotePatch } from '$lib/ipc/types';

class NotesStore {
  notes = $state<Note[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<string | null>(null);
  private subscribed = false;
  private suppressNextEvent = false;

  get selected(): Note | null {
    if (!this.selectedId) return null;
    return this.notes.find((n) => n.id === this.selectedId) ?? null;
  }

  /** Wire up the cross-window event listener exactly once. */
  async ensureSubscribed(): Promise<void> {
    if (this.subscribed) return;
    this.subscribed = true;
    try {
      await on('notes:changed', () => {
        // Skip the immediate echo from a mutation we just performed locally;
        // we already merged the response into state.
        if (this.suppressNextEvent) {
          this.suppressNextEvent = false;
          return;
        }
        void this.refresh();
      });
    } catch (e) {
      this.subscribed = false;
      console.warn('failed to subscribe to notes:changed', e);
    }
  }

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.notes = await api.listNotes();
      if (this.selectedId && !this.notes.some((n) => n.id === this.selectedId)) {
        this.selectedId = this.notes[0]?.id ?? null;
      }
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  }

  async create(): Promise<Note | null> {
    this.error = null;
    try {
      this.suppressNextEvent = true;
      const note = await api.createNote({ title: '', body: '' });
      this.notes = [note, ...this.notes];
      this.selectedId = note.id;
      return note;
    } catch (e) {
      this.error = errorMessage(e);
      return null;
    }
  }

  async update(id: string, patch: NotePatch): Promise<void> {
    this.error = null;
    try {
      this.suppressNextEvent = true;
      const updated = await api.updateNote(id, patch);
      this.notes = this.notes.map((n) => (n.id === id ? updated : n));
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  async remove(id: string): Promise<void> {
    this.error = null;
    try {
      this.suppressNextEvent = true;
      await api.deleteNote(id);
      this.notes = this.notes.filter((n) => n.id !== id);
      if (this.selectedId === id) {
        this.selectedId = this.notes[0]?.id ?? null;
      }
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  select(id: string | null): void {
    this.selectedId = id;
  }
}

function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

export const notesStore = new NotesStore();
