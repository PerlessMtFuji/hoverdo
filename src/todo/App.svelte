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

<!-- Frosted panel: neutral semi-transparent backdrop, no accent tint in the
     body. The kind identity comes only from the thin ribbon + the add-button
     gradient — not from a heavy colored surface. -->
<div class="frosted-panel todo-widget flex h-full flex-col overflow-hidden" style="opacity: {opacity};">

  <!-- Thin violet→indigo ribbon: the only place we use the kind accent. -->
  <span
    aria-hidden="true"
    class="h-1 w-full shrink-0"
    style="background: linear-gradient(90deg, #9d7bff 0%, #7ba0ff 100%);"
  ></span>

  <!-- Drag strip + title + close. Kept minimal so chrome disappears and the
       content is the focus. -->
  <header
    class="drag-region flex h-9 shrink-0 items-center gap-1.5 px-3"
  >
    <input
      type="text"
      data-no-drag
      bind:value={titleDraft}
      oninput={scheduleTitleSave}
      onblur={flushTitleSave}
      placeholder="Untitled list"
      class="min-w-0 flex-1 border-0 bg-transparent text-sm font-semibold text-text-1 placeholder:text-text-3"
    />
    {#if remainingCount() > 0}
      <span
        class="shrink-0 rounded-pill px-1.5 py-0.5 text-[10px] tabular-nums"
        style="background: rgba(157,123,255,0.18); color: #9d7bff;"
      >
        {remainingCount()}
      </span>
    {/if}
    <button
      type="button"
      data-no-drag
      class="grid h-6 w-6 shrink-0 place-items-center rounded-control text-text-3 transition-colors hover:bg-danger/10 hover:text-danger"
      aria-label="Unpin and close"
      onclick={handleClose}
    >
      <X size={13} />
    </button>
  </header>

  {#if ready}
    <ul class="flex-1 overflow-y-auto px-1.5 py-1">
      {#if tasks.length === 0}
        <li class="px-2 py-6 text-center text-xs text-text-3">
          Nothing yet — add your first task below.
        </li>
      {:else}
        {#each tasks as task (task.id)}
          <li
            class="group flex items-center gap-2 rounded-control px-2 py-1.5 text-sm transition-colors hover:bg-border-subtle"
          >
            <!-- Custom checkbox: circle that fills with the accent gradient on check. -->
            <button
              type="button"
              class="todo-check grid h-4 w-4 shrink-0 place-items-center rounded-pill transition-all"
              class:todo-check--done={task.done}
              aria-checked={task.done}
              role="checkbox"
              onclick={() => toggle(task.id, !task.done)}
            >
              {#if task.done}
                <Check size={10} />
              {/if}
            </button>
            <span
              class="flex-1 truncate text-[13px]"
              class:line-through={task.done}
              class:opacity-40={task.done}
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
              <X size={10} />
            </button>
          </li>
        {/each}
      {/if}
    </ul>

    <!-- Add task bar: transparent input to match the frosted body, gradient
         submit pill so the CTA is clear without being loud. -->
    <form
      onsubmit={add}
      class="flex shrink-0 items-center gap-1.5 border-t border-border-subtle px-2 py-1.5"
    >
      <input
        type="text"
        bind:value={draft}
        placeholder="Add task…"
        class="flex-1 border-0 bg-transparent text-[13px] text-text-1 placeholder:text-text-3"
      />
      <button
        type="submit"
        class="grid h-6 w-6 place-items-center rounded-pill text-white transition-transform hover:scale-105 disabled:opacity-40"
        style="background: linear-gradient(135deg, #9d7bff 0%, #7ba0ff 100%);"
        aria-label="Add task"
        disabled={!draft.trim()}
      >
        <Plus size={13} />
      </button>
    </form>
    <div class="px-1.5 pb-1.5">
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

<style>
  /* Custom round checkbox: unchecked = hollow circle outline;
     checked = gradient-filled circle with a white tick. */
  .todo-check {
    border: 1.5px solid rgba(157, 123, 255, 0.45);
    background: transparent;
    color: transparent;
    cursor: pointer;
    transition: border-color 150ms, background 150ms, color 150ms;
  }
  .todo-check:hover {
    border-color: rgba(157, 123, 255, 0.75);
  }
  .todo-check--done {
    background: linear-gradient(135deg, #9d7bff 0%, #7ba0ff 100%);
    border-color: transparent;
    color: #ffffff;
  }

  /* Ensure the frosted-panel fills the full window and clips children to
     its border-radius, so the accent ribbon and other children are also
     rounded at the top. */
  .todo-widget {
    overflow: hidden;
  }
</style>
