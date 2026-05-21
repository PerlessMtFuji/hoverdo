<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { X, Pin } from '@lucide/svelte';

  import * as notesApi from '$lib/ipc/commands';
  import * as widgetsApi from '$lib/ipc/widgets';
  import { on } from '$lib/ipc/events';
  import type { Note } from '$lib/ipc/types';
  import type { WidgetInstance } from '$lib/ipc/widgets';
  import WidgetSettings from '$lib/components/WidgetSettings.svelte';

  // Widgets receive their target via URL: sticky.html?widget_id=<uuid>
  const widgetId =
    new URLSearchParams(window.location.search).get('widget_id') ?? '';

  let widget = $state<WidgetInstance | null>(null);
  let note = $state<Note | null>(null);
  let draftBody = $state('');
  let saving = $state(false);
  let ready = $state(false);
  let opacity = $state(1);
  let alwaysOnTop = $state(false);

  const BODY_DEBOUNCE_MS = 500;
  const GEOM_DEBOUNCE_MS = 400;
  const OPACITY_DEBOUNCE_MS = 200;
  let bodyTimer: number | null = null;
  let geomTimer: number | null = null;
  let opacityTimer: number | null = null;

  let unsubs: Array<() => void> = [];
  const win = getCurrentWindow();

  async function load() {
    if (!widgetId) return;
    widget = await widgetsApi.getWidget(widgetId);
    if (!widget) {
      // Instance was unpinned remotely; close this window.
      await win.close();
      return;
    }
    note = await notesApi.getNote(widget.target_id);
    if (!note) {
      // Underlying note was deleted - unpin and close.
      await widgetsApi.unpinWidget(widgetId);
      await win.close();
      return;
    }
    draftBody = note.body;
    opacity = widget.opacity ?? 1;
    alwaysOnTop = widget.always_on_top ?? false;
    // Native always-on-top is applied at spawn time; nothing to do here.
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

  function scheduleBodySave() {
    if (bodyTimer !== null) window.clearTimeout(bodyTimer);
    bodyTimer = window.setTimeout(flushBodySave, BODY_DEBOUNCE_MS);
  }

  async function flushBodySave() {
    if (bodyTimer !== null) {
      window.clearTimeout(bodyTimer);
      bodyTimer = null;
    }
    if (!note) return;
    if (draftBody === note.body) return;
    saving = true;
    try {
      const updated = await notesApi.updateNote(note.id, { body: draftBody });
      note = updated;
    } catch (e) {
      console.warn('sticky save failed', e);
    } finally {
      saving = false;
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
    await flushBodySave();
    await win.close();
  }

  onMount(async () => {
    await load();

    const offMoved = await win.onMoved(() => scheduleGeomSave());
    const offResized = await win.onResized(() => scheduleGeomSave());
    unsubs.push(offMoved, offResized);

    const offNotes = await on('notes:changed', async (changedId) => {
      if (!widget || changedId !== widget.target_id) return;
      // External edit while we're holding a draft: only refresh if the user
      // is currently in sync with the backend. Otherwise keep the draft -
      // the next debounced save resolves the conflict last-writer-wins.
      const fresh = await notesApi.getNote(widget.target_id);
      if (!fresh) {
        await win.close();
        return;
      }
      if (note && draftBody === note.body) {
        note = fresh;
        draftBody = fresh.body;
      } else {
        note = fresh;
      }
    });
    unsubs.push(offNotes);
  });

  onDestroy(() => {
    void flushBodySave();
    if (bodyTimer !== null) window.clearTimeout(bodyTimer);
    if (geomTimer !== null) window.clearTimeout(geomTimer);
    if (opacityTimer !== null) window.clearTimeout(opacityTimer);
    for (const u of unsubs) u();
  });

  function previewTitle(): string {
    const t = note?.title?.trim() ?? '';
    return t.length > 0 ? t : 'Untitled';
  }
</script>

<div class="flex h-full flex-col" style="opacity: {opacity};">
  <header
    class="drag-region flex h-7 shrink-0 items-center gap-1.5 border-b border-border-subtle px-2 text-xs text-text-3"
  >
    <Pin size={11} class="shrink-0 opacity-60" />
    <span class="flex-1 truncate text-text-2">{previewTitle()}</span>
    <span class="shrink-0 tabular-nums">
      {#if saving}saving…{/if}
    </span>
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
    <textarea
      bind:value={draftBody}
      oninput={scheduleBodySave}
      onblur={flushBodySave}
      placeholder="Write a quick thought…"
      class="flex-1 resize-none border-0 bg-transparent p-3 text-sm leading-relaxed text-text-1 placeholder:text-text-3 focus:outline-none"
    ></textarea>
    <WidgetSettings
      {opacity}
      {alwaysOnTop}
      onOpacityChange={applyOpacity}
      onAlwaysOnTopChange={applyAlwaysOnTop}
    />
  {:else}
    <div class="m-auto text-xs text-text-3">Loading…</div>
  {/if}
</div>
