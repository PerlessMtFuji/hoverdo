<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { X, ListChecks, Plus } from '@lucide/svelte';

  import * as api from '$lib/ipc/commands';
  import * as widgetsApi from '$lib/ipc/widgets';
  import { on } from '$lib/ipc/events';
  import type { List, Task } from '$lib/ipc/types';
  import type { WidgetInstance } from '$lib/ipc/widgets';

  const widgetId =
    new URLSearchParams(window.location.search).get('widget_id') ?? '';

  let widget = $state<WidgetInstance | null>(null);
  let list = $state<List | null>(null);
  let tasks = $state<Task[]>([]);
  let draft = $state('');
  let ready = $state(false);

  const GEOM_DEBOUNCE_MS = 400;
  let geomTimer: number | null = null;

  let unsubs: Array<() => void> = [];
  const win = getCurrentWindow();

  async function load() {
    if (!widgetId) return;
    widget = await widgetsApi.getWidget(widgetId);
    if (!widget) {
      await win.close();
      return;
    }
    list = await api.getList(widget.target_id);
    if (!list) {
      // Underlying list was deleted - unpin and close.
      await widgetsApi.unpinWidget(widgetId);
      await win.close();
      return;
    }
    tasks = await api.listTasks(widget.target_id);
    ready = true;
  }

  async function add(e: SubmitEvent) {
    e.preventDefault();
    if (!widget || !draft.trim()) return;
    try {
      const t = await api.createTask({ list_id: widget.target_id, title: draft });
      tasks = [...tasks, t];
      draft = '';
    } catch (err) {
      console.warn('add task failed', err);
    }
  }

  async function toggle(id: string, done: boolean) {
    try {
      const updated = await api.setTaskDone(id, done);
      tasks = tasks.map((t) => (t.id === id ? updated : t));
    } catch (err) {
      console.warn('toggle failed', err);
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteTask(id);
      tasks = tasks.filter((t) => t.id !== id);
    } catch (err) {
      console.warn('delete failed', err);
    }
  }

  function scheduleGeomSave() {
    if (geomTimer !== null) window.clearTimeout(geomTimer);
    geomTimer = window.setTimeout(flushGeomSave, GEOM_DEBOUNCE_MS);
  }

  async function flushGeomSave() {
    if (geomTimer !== null) {
      window.clearTimeout(geomTimer);
      geomTimer = null;
    }
    if (!widget) return;
    try {
      const pos = await win.outerPosition();
      const size = await win.outerSize();
      await widgetsApi.saveWidgetGeometry(
        widget.id,
        pos.x,
        pos.y,
        size.width,
        size.height
      );
    } catch (e) {
      console.warn('geometry save failed', e);
    }
  }

  async function handleClose() {
    await win.close();
  }

  onMount(async () => {
    await load();

    const offMoved = await win.onMoved(() => scheduleGeomSave());
    const offResized = await win.onResized(() => scheduleGeomSave());
    unsubs.push(offMoved, offResized);

    const offTasks = await on('tasks:changed', async (changedListId) => {
      if (!widget || changedListId !== widget.target_id) return;
      tasks = await api.listTasks(widget.target_id);
    });
    const offLists = await on('lists:changed', async (changedListId) => {
      if (!widget || changedListId !== widget.target_id) return;
      const fresh = await api.getList(widget.target_id);
      if (!fresh) {
        await win.close();
        return;
      }
      list = fresh;
    });
    unsubs.push(offTasks, offLists);
  });

  onDestroy(() => {
    if (geomTimer !== null) window.clearTimeout(geomTimer);
    for (const u of unsubs) u();
  });

  function previewTitle(): string {
    const t = list?.title?.trim() ?? '';
    return t.length > 0 ? t : 'Untitled list';
  }
</script>

<div class="flex h-full flex-col">
  <header
    class="drag-region flex h-7 shrink-0 items-center gap-1.5 border-b border-border-subtle px-2 text-xs text-text-3"
  >
    <ListChecks size={11} class="shrink-0 opacity-60" />
    <span class="flex-1 truncate text-text-2">{previewTitle()}</span>
    <button
      type="button"
      class="grid h-5 w-5 place-items-center rounded-control text-text-3 transition-colors hover:bg-danger/10 hover:text-danger"
      aria-label="Unpin and close"
      data-no-drag
      onclick={handleClose}
    >
      <X size={12} />
    </button>
  </header>

  {#if ready}
    <ul class="flex-1 overflow-y-auto px-2 py-1.5">
      {#if tasks.length === 0}
        <li class="px-2 py-3 text-center text-xs text-text-3">
          No tasks yet.
        </li>
      {:else}
        {#each tasks as task (task.id)}
          <li
            class="group flex items-center gap-2 rounded-control px-2 py-1 text-sm hover:bg-surface-2"
          >
            <input
              type="checkbox"
              checked={task.done}
              onchange={(e) =>
                toggle(task.id, (e.currentTarget as HTMLInputElement).checked)}
              class="size-4 accent-accent"
            />
            <span
              class="flex-1"
              class:line-through={task.done}
              class:text-text-3={task.done}
            >
              {task.title}
            </span>
            <button
              type="button"
              class="rounded-control p-0.5 text-text-3 opacity-0 transition-opacity hover:text-danger group-hover:opacity-100"
              aria-label="Delete task"
              onclick={() => remove(task.id)}
            >
              <X size={11} />
            </button>
          </li>
        {/each}
      {/if}
    </ul>

    <form
      onsubmit={add}
      class="flex shrink-0 items-center gap-1 border-t border-border-subtle px-2 py-2"
    >
      <input
        type="text"
        bind:value={draft}
        placeholder="Add task…"
        class="flex-1 rounded-control bg-surface-1 px-2 py-1 text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:ring-1 focus:ring-accent"
      />
      <button
        type="submit"
        class="grid h-7 w-7 place-items-center rounded-control text-text-2 transition-colors hover:bg-surface-2 hover:text-text-1"
        aria-label="Add"
      >
        <Plus size={14} />
      </button>
    </form>
  {:else}
    <div class="m-auto text-xs text-text-3">Loading…</div>
  {/if}
</div>
