<script lang="ts">
  // To-do list widget.
  //
  // Visual model: a glass card that picks up the active theme. Top edge
  // carries a violet→indigo gradient strip as the widget's brand signature
  // (matches the to-do tile in the home hub). Title is inline-editable;
  // tasks are a tight checklist with hover-revealed delete.

  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { X, Plus, Check } from '@lucide/svelte';

  import * as api from '$lib/ipc/commands';
  import * as widgetsApi from '$lib/ipc/widgets';
  import { on } from '$lib/ipc/events';
  import type { List, Task } from '$lib/ipc/types';
  import type { WidgetInstance } from '$lib/ipc/widgets';
  import WidgetSettings from '$lib/components/WidgetSettings.svelte';

  const widgetId =
    new URLSearchParams(window.location.search).get('widget_id') ?? '';

  let widget = $state<WidgetInstance | null>(null);
  let list = $state<List | null>(null);
  let tasks = $state<Task[]>([]);
  let draft = $state('');
  let titleDraft = $state('');
  let ready = $state(false);
  let opacity = $state(1);
  let alwaysOnTop = $state(false);

  const GEOM_DEBOUNCE_MS = 400;
  const TITLE_DEBOUNCE_MS = 400;
  const OPACITY_DEBOUNCE_MS = 200;
  let geomTimer: number | null = null;
  let titleTimer: number | null = null;
  let opacityTimer: number | null = null;

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
      await widgetsApi.unpinWidget(widgetId);
      await win.close();
      return;
    }
    tasks = await api.listTasks(widget.target_id);
    titleDraft = list.title;
    opacity = widget.opacity ?? 1;
    alwaysOnTop = widget.always_on_top ?? false;
    ready = true;
  }

  function applyOpacity(value: number) {
    opacity = value;
    if (!widget) return;
    if (opacityTimer !== null) window.clearTimeout(opacityTimer);
    opacityTimer = window.setTimeout(() => {
      void widgetsApi.setWidgetOpacity(widget!.id, value);
    }, OPACITY_DEBOUNCE_MS);
  }

  async function applyAlwaysOnTop(value: boolean) {
    alwaysOnTop = value;
    if (!widget) return;
    try {
      await widgetsApi.setWidgetAlwaysOnTop(widget.id, value);
    } catch (e) {
      console.warn('set always-on-top failed', e);
    }
  }

  function scheduleTitleSave() {
    if (titleTimer !== null) window.clearTimeout(titleTimer);
    titleTimer = window.setTimeout(flushTitleSave, TITLE_DEBOUNCE_MS);
  }

  async function flushTitleSave() {
    if (titleTimer !== null) {
      window.clearTimeout(titleTimer);
      titleTimer = null;
    }
    if (!list) return;
    const next = titleDraft.trim() || 'Untitled list';
    if (next === list.title) return;
    try {
      const updated = await api.renameList(list.id, next);
      list = updated;
    } catch (e) {
      console.warn('rename failed', e);
    }
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
    await flushTitleSave();
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
      if (list && titleDraft === list.title) titleDraft = fresh.title;
      list = fresh;
    });
    unsubs.push(offTasks, offLists);
  });

  onDestroy(() => {
    if (geomTimer !== null) window.clearTimeout(geomTimer);
    if (titleTimer !== null) window.clearTimeout(titleTimer);
    if (opacityTimer !== null) window.clearTimeout(opacityTimer);
    void flushTitleSave();
    for (const u of unsubs) u();
  });

  function remainingCount(): number {
    return tasks.filter((t) => !t.done).length;
  }
</script>

<!-- Glass card body. Top accent strip is the kind signature; we keep the
     full window borderless so the rounded card shape shows through. -->
<div
  class="glass-card-strong flex h-full flex-col overflow-hidden"
  style="opacity: {opacity};"
>
  <!-- Accent ribbon -->
  <span
    aria-hidden="true"
    class="h-1.5 w-full shrink-0"
    style="background: linear-gradient(90deg, #8c6bff 0%, #6b8cff 100%);"
  ></span>

  <header
    class="drag-region flex h-10 shrink-0 items-center gap-1.5 px-3 text-xs text-text-3"
  >
    <span
      aria-hidden="true"
      class="grid h-5 w-5 place-items-center rounded-pill text-text-on-brand shadow-hd-sm"
      style="background: linear-gradient(135deg, #8c6bff 0%, #6b8cff 100%);"
    >
      <Check size={11} />
    </span>
    <input
      type="text"
      data-no-drag
      bind:value={titleDraft}
      oninput={scheduleTitleSave}
      onblur={flushTitleSave}
      placeholder="Untitled list"
      class="min-w-0 flex-1 border-0 bg-transparent text-sm font-semibold text-text-1 placeholder:text-text-3 focus:outline-none"
    />
    {#if remainingCount() > 0}
      <span class="shrink-0 rounded-pill bg-surface-2 px-1.5 py-0.5 text-[10px] tabular-nums text-text-2">
        {remainingCount()}
      </span>
    {/if}
    <button
      type="button"
      data-no-drag
      class="grid h-6 w-6 place-items-center rounded-control text-text-3 transition-colors hover:bg-danger/10 hover:text-danger"
      aria-label="Unpin and close"
      onclick={handleClose}
    >
      <X size={13} />
    </button>
  </header>

  {#if ready}
    <ul class="flex-1 overflow-y-auto px-2 py-1">
      {#if tasks.length === 0}
        <li class="px-2 py-6 text-center text-xs text-text-3">
          Nothing yet — add your first task below.
        </li>
      {:else}
        {#each tasks as task (task.id)}
          <li
            class="group flex items-center gap-2 rounded-control px-2 py-1.5 text-sm transition-colors hover:bg-surface-2"
          >
            <input
              type="checkbox"
              checked={task.done}
              onchange={(e) =>
                toggle(task.id, (e.currentTarget as HTMLInputElement).checked)}
              class="size-4 cursor-pointer accent-accent"
            />
            <span
              class="flex-1 truncate"
              class:line-through={task.done}
              class:text-text-3={task.done}
              class:text-text-1={!task.done}
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
      class="flex shrink-0 items-center gap-1 border-t border-border-subtle bg-surface-sunken px-2 py-2"
    >
      <input
        type="text"
        bind:value={draft}
        placeholder="Add task…"
        class="flex-1 rounded-control border border-border-subtle bg-surface-1 px-2 py-1 text-sm text-text-1 placeholder:text-text-3 focus:border-transparent focus:outline-none focus:ring-2 focus:ring-accent/40"
      />
      <button
        type="submit"
        class="grid h-7 w-7 place-items-center rounded-control text-text-on-brand transition-transform hover:scale-105"
        style="background: linear-gradient(135deg, #8c6bff 0%, #6b8cff 100%);"
        aria-label="Add task"
        disabled={!draft.trim()}
      >
        <Plus size={14} />
      </button>
    </form>
    <div class="px-2 pb-2">
      <WidgetSettings
        {opacity}
        {alwaysOnTop}
        onOpacityChange={applyOpacity}
        onAlwaysOnTopChange={applyAlwaysOnTop}
      />
    </div>
  {:else}
    <div class="m-auto text-xs text-text-3">Loading…</div>
  {/if}
</div>
