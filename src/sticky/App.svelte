<script lang="ts">
  // Sticky note widget.
  //
  // Visual model: a colored paper card with a subtle gradient, soft drop
  // shadow, top edge highlight. Chrome is minimal - just an invisible drag
  // strip across the top, with a tiny close + colour picker that appear
  // on hover. The body is an editable textarea; the title is editable
  // inline above it.
  //
  // Per-note color persists to the `notes.color` field; the swatch list is
  // a small fixed palette (var(--hd-paper-*)) so widgets stay legible no
  // matter the theme.

  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { X, Palette } from '@lucide/svelte';

  import * as notesApi from '$lib/ipc/commands';
  import * as widgetsApi from '$lib/ipc/widgets';
  import { on } from '$lib/ipc/events';
  import type { Note } from '$lib/ipc/types';
  import type { WidgetInstance } from '$lib/ipc/widgets';
  import WidgetSettings from '$lib/components/WidgetSettings.svelte';

  const widgetId =
    new URLSearchParams(window.location.search).get('widget_id') ?? '';

  let widget = $state<WidgetInstance | null>(null);
  let note = $state<Note | null>(null);
  let draftBody = $state('');
  let draftTitle = $state('');
  let saving = $state(false);
  let ready = $state(false);
  let opacity = $state(1);
  let alwaysOnTop = $state(false);
  let paletteOpen = $state(false);

  const BODY_DEBOUNCE_MS = 500;
  const TITLE_DEBOUNCE_MS = 400;
  const GEOM_DEBOUNCE_MS = 400;
  const OPACITY_DEBOUNCE_MS = 200;
  let bodyTimer: number | null = null;
  let titleTimer: number | null = null;
  let geomTimer: number | null = null;
  let opacityTimer: number | null = null;

  // Paper palette mirrors --hd-paper-* tokens. Stored as CSS vars so the
  // sticky picks up whatever the active theme defines for that swatch.
  const swatches: { id: string; var: string; label: string }[] = [
    { id: 'butter', var: '--hd-paper-butter', label: 'Butter' },
    { id: 'peach', var: '--hd-paper-peach', label: 'Peach' },
    { id: 'rose', var: '--hd-paper-rose', label: 'Rose' },
    { id: 'lilac', var: '--hd-paper-lilac', label: 'Lilac' },
    { id: 'mint', var: '--hd-paper-mint', label: 'Mint' },
    { id: 'sky', var: '--hd-paper-sky', label: 'Sky' },
    { id: 'sand', var: '--hd-paper-sand', label: 'Sand' }
  ];

  function swatchVarFor(id: string | null | undefined): string {
    const found = swatches.find((s) => s.id === id);
    return found ? `var(${found.var})` : 'var(--hd-paper-butter)';
  }

  let unsubs: Array<() => void> = [];
  const win = getCurrentWindow();

  async function load() {
    if (!widgetId) return;
    widget = await widgetsApi.getWidget(widgetId);
    if (!widget) {
      await win.close();
      return;
    }
    note = await notesApi.getNote(widget.target_id);
    if (!note) {
      await widgetsApi.unpinWidget(widgetId);
      await win.close();
      return;
    }
    draftBody = note.body;
    draftTitle = note.title;
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
      console.warn('sticky body save failed', e);
    } finally {
      saving = false;
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
    if (!note) return;
    if (draftTitle === note.title) return;
    saving = true;
    try {
      const updated = await notesApi.updateNote(note.id, { title: draftTitle });
      note = updated;
    } catch (e) {
      console.warn('sticky title save failed', e);
    } finally {
      saving = false;
    }
  }

  async function setColor(id: string) {
    if (!note) return;
    paletteOpen = false;
    try {
      const updated = await notesApi.updateNote(note.id, { color: id });
      note = updated;
    } catch (e) {
      console.warn('color save failed', e);
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
    await flushTitleSave();
    await win.close();
  }

  onMount(async () => {
    await load();

    const offMoved = await win.onMoved(() => scheduleGeomSave());
    const offResized = await win.onResized(() => scheduleGeomSave());
    unsubs.push(offMoved, offResized);

    const offNotes = await on('notes:changed', async (changedId) => {
      if (!widget || changedId !== widget.target_id) return;
      const fresh = await notesApi.getNote(widget.target_id);
      if (!fresh) {
        await win.close();
        return;
      }
      // Only adopt remote drafts if the local one is in sync; otherwise the
      // user's typing wins until they pause and our debounced save flushes.
      if (note && draftBody === note.body) {
        draftBody = fresh.body;
      }
      if (note && draftTitle === note.title) {
        draftTitle = fresh.title;
      }
      note = fresh;
    });
    unsubs.push(offNotes);
  });

  onDestroy(() => {
    void flushBodySave();
    void flushTitleSave();
    if (bodyTimer !== null) window.clearTimeout(bodyTimer);
    if (titleTimer !== null) window.clearTimeout(titleTimer);
    if (geomTimer !== null) window.clearTimeout(geomTimer);
    if (opacityTimer !== null) window.clearTimeout(opacityTimer);
    for (const u of unsubs) u();
  });
</script>

<!-- Widget root: full-window paper card. The transparent Tauri window means
     the rounded paper edges show through; the soft shadow simulates the
     card sitting above the desktop. -->
<div
  class="paper relative flex h-full flex-col"
  style="--hd-paper-current: {swatchVarFor(note?.color)}; opacity: {opacity};"
>
  <!-- Drag strip + chrome. Invisible by default, surfaces on hover so the
       sticky feels like a clean piece of paper at rest. -->
  <header
    class="drag-region group/chrome relative flex h-9 shrink-0 items-center gap-1 px-3"
  >
    <span
      aria-hidden="true"
      class="pointer-events-none absolute left-3 top-3 h-1 w-10 rounded-pill bg-black/15"
    ></span>
    <span class="flex-1"></span>

    <span
      class="hidden text-[10px] uppercase tracking-wider text-black/45 tabular-nums sm:inline group-hover/chrome:inline"
    >
      {#if saving}Saving…{/if}
    </span>

    <div class="relative" data-no-drag>
      <button
        type="button"
        class="grid h-6 w-6 place-items-center rounded-pill text-black/55 transition-colors hover:bg-black/10 hover:text-black"
        aria-label="Change colour"
        title="Change colour"
        onclick={() => (paletteOpen = !paletteOpen)}
      >
        <Palette size={12} />
      </button>
      {#if paletteOpen}
        <div
          class="absolute right-0 top-7 z-10 flex gap-1 rounded-pill bg-black/65 p-1.5 shadow-hd-md backdrop-blur"
          role="dialog"
        >
          {#each swatches as sw (sw.id)}
            <button
              type="button"
              aria-label={sw.label}
              title={sw.label}
              onclick={() => setColor(sw.id)}
              class="h-4 w-4 rounded-pill ring-1 ring-white/30 transition-transform hover:scale-110"
              class:ring-2={note?.color === sw.id}
              style="background: var({sw.var});"
            ></button>
          {/each}
        </div>
      {/if}
    </div>

    <button
      type="button"
      data-no-drag
      class="grid h-6 w-6 place-items-center rounded-pill text-black/55 transition-colors hover:bg-black/10 hover:text-black"
      aria-label="Unpin and close"
      title="Unpin and close"
      onclick={handleClose}
    >
      <X size={13} />
    </button>
  </header>

  {#if ready}
    <input
      type="text"
      bind:value={draftTitle}
      oninput={scheduleTitleSave}
      onblur={flushTitleSave}
      placeholder="Untitled"
      class="border-0 bg-transparent px-4 pb-1 pt-0 text-base font-semibold text-black placeholder:text-black/35 focus:outline-none"
    />
    <textarea
      bind:value={draftBody}
      oninput={scheduleBodySave}
      onblur={flushBodySave}
      placeholder="Write a quick thought…"
      class="flex-1 resize-none border-0 bg-transparent px-4 pb-3 pt-1 text-[13.5px] leading-relaxed text-black/85 placeholder:text-black/40 focus:outline-none"
    ></textarea>
    <div class="px-2 pb-2">
      <WidgetSettings
        {opacity}
        {alwaysOnTop}
        onOpacityChange={applyOpacity}
        onAlwaysOnTopChange={applyAlwaysOnTop}
        tone="paper"
      />
    </div>
  {:else}
    <div class="m-auto text-xs text-black/50">Loading…</div>
  {/if}
</div>
