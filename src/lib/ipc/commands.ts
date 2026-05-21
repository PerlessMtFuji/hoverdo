// Typed wrappers around Tauri's `invoke`. Importers never call `invoke`
// directly so the IPC surface stays auditable.

import { invoke } from '@tauri-apps/api/core';
import type {
  CreateTaskInput,
  Health,
  List,
  NewList,
  NewNote,
  Note,
  NotePatch,
  Reminder,
  Task
} from './types';

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

// ---- lists --------------------------------------------------------------

export function createList(input: NewList): Promise<List> {
  return invoke('create_list', { input });
}

export function listLists(): Promise<List[]> {
  return invoke('list_lists');
}

export function getList(id: string): Promise<List | null> {
  return invoke('get_list', { id });
}

export function renameList(id: string, title: string): Promise<List> {
  return invoke('rename_list', { id, title });
}

export function deleteList(id: string): Promise<void> {
  return invoke('delete_list', { id });
}

// ---- tasks --------------------------------------------------------------

export function createTask(input: CreateTaskInput): Promise<Task> {
  return invoke('create_task', { input });
}

export function listTasks(listId: string): Promise<Task[]> {
  return invoke('list_tasks', { listId });
}

export function setTaskDone(id: string, done: boolean): Promise<Task> {
  return invoke('set_task_done', { id, done });
}

export function deleteTask(id: string): Promise<void> {
  return invoke('delete_task', { id });
}

// ---- reminders ----------------------------------------------------------

export function setTaskReminder(taskId: string, dueAt: string): Promise<Reminder> {
  // Backend wraps both fields in a single `input` payload so `due_at` can opt
  // into RFC3339 serde (time's default would expect an array).
  return invoke('set_task_reminder', { input: { task_id: taskId, due_at: dueAt } });
}

export function cancelReminder(id: string): Promise<void> {
  return invoke('cancel_reminder', { id });
}

export function listRemindersForList(listId: string): Promise<Reminder[]> {
  return invoke('list_reminders_for_list', { listId });
}
