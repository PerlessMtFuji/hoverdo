// Typed wrappers around Tauri's `invoke`. Importers never call `invoke`
// directly so the IPC surface stays auditable.

import { invoke } from '@tauri-apps/api/core';
import type { Health, NewNote, Note, NotePatch } from './types';

export function ping(): Promise<string> {
  return invoke('ping');
}

export function health(): Promise<Health> {
  return invoke('health');
}

export function createNote(input: NewNote): Promise<Note> {
  return invoke('create_note', { input });
}

export function listNotes(): Promise<Note[]> {
  return invoke('list_notes');
}

export function getNote(id: string): Promise<Note | null> {
  return invoke('get_note', { id });
}

export function updateNote(id: string, patch: NotePatch): Promise<Note> {
  return invoke('update_note', { id, patch });
}

export function deleteNote(id: string): Promise<void> {
  return invoke('delete_note', { id });
}
