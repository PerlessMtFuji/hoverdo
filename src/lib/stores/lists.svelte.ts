// Lists + tasks store. Single class so the UI doesn't have to coordinate
// two reactive sources when switching between lists - we always know
// which list is selected and which tasks belong to it.

import * as api from '$lib/ipc/commands';
import { on } from '$lib/ipc/events';
import type { List, Task } from '$lib/ipc/types';

class ListsStore {
  lists = $state<List[]>([]);
  tasks = $state<Task[]>([]);
  selectedId = $state<string | null>(null);
  loading = $state(false);
  loadingTasks = $state(false);
  error = $state<string | null>(null);
  private subscribed = false;
  private suppressListsEvent = false;
  private suppressTasksEvent = false;

  get selected(): List | null {
    if (!this.selectedId) return null;
    return this.lists.find((l) => l.id === this.selectedId) ?? null;
  }

  async ensureSubscribed(): Promise<void> {
    if (this.subscribed) return;
    this.subscribed = true;
    try {
      await on('lists:changed', () => {
        if (this.suppressListsEvent) {
          this.suppressListsEvent = false;
          return;
        }
        void this.refresh();
      });
      await on('tasks:changed', (listId) => {
        if (this.suppressTasksEvent) {
          this.suppressTasksEvent = false;
          return;
        }
        if (listId === this.selectedId) void this.refreshTasks();
      });
    } catch (e) {
      this.subscribed = false;
      console.warn('failed to subscribe to lists/tasks events', e);
    }
  }

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.lists = await api.listLists();
      if (this.selectedId && !this.lists.some((l) => l.id === this.selectedId)) {
        this.selectedId = this.lists[0]?.id ?? null;
        await this.refreshTasks();
      } else if (!this.selectedId && this.lists.length > 0) {
        this.selectedId = this.lists[0].id;
        await this.refreshTasks();
      }
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  }

  async refreshTasks(): Promise<void> {
    if (!this.selectedId) {
      this.tasks = [];
      return;
    }
    this.loadingTasks = true;
    try {
      this.tasks = await api.listTasks(this.selectedId);
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loadingTasks = false;
    }
  }

  async select(id: string | null): Promise<void> {
    if (id === this.selectedId) return;
    this.selectedId = id;
    this.tasks = [];
    if (id) await this.refreshTasks();
  }

  async create(): Promise<List | null> {
    this.error = null;
    try {
      this.suppressListsEvent = true;
      const list = await api.createList({ title: 'New list' });
      this.lists = [list, ...this.lists];
      this.selectedId = list.id;
      this.tasks = [];
      return list;
    } catch (e) {
      this.error = errorMessage(e);
      return null;
    }
  }

  async rename(id: string, title: string): Promise<void> {
    this.error = null;
    try {
      this.suppressListsEvent = true;
      const updated = await api.renameList(id, title);
      this.lists = this.lists.map((l) => (l.id === id ? updated : l));
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  async remove(id: string): Promise<void> {
    this.error = null;
    try {
      this.suppressListsEvent = true;
      await api.deleteList(id);
      this.lists = this.lists.filter((l) => l.id !== id);
      if (this.selectedId === id) {
        this.selectedId = this.lists[0]?.id ?? null;
        this.tasks = [];
        if (this.selectedId) await this.refreshTasks();
      }
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  // ---- tasks --------------------------------------------------------

  async addTask(title: string): Promise<Task | null> {
    if (!this.selectedId) return null;
    const trimmed = title.trim();
    if (!trimmed) return null;
    this.error = null;
    try {
      this.suppressTasksEvent = true;
      const task = await api.createTask({
        list_id: this.selectedId,
        title: trimmed
      });
      this.tasks = [...this.tasks, task];
      return task;
    } catch (e) {
      this.error = errorMessage(e);
      return null;
    }
  }

  async toggleTask(id: string, done: boolean): Promise<void> {
    this.error = null;
    try {
      this.suppressTasksEvent = true;
      const updated = await api.setTaskDone(id, done);
      this.tasks = this.tasks.map((t) => (t.id === id ? updated : t));
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  async removeTask(id: string): Promise<void> {
    this.error = null;
    try {
      this.suppressTasksEvent = true;
      await api.deleteTask(id);
      this.tasks = this.tasks.filter((t) => t.id !== id);
    } catch (e) {
      this.error = errorMessage(e);
    }
  }
}

function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

export const listsStore = new ListsStore();
