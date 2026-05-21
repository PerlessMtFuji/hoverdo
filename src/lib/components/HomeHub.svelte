<script lang="ts">
  // The widget hub - landing surface that makes "create / manage floating
  // widgets" the primary affordance instead of burying it inside the notes
  // library. Layout:
  //
  //   Hero greeting + brand mark
  //   ─── Spawn tiles row (Sticky Note / To-do List) ───────────
  //   ─── Pinned gallery (active widget thumbnails) ────────────
  //   ─── Recent notes / recent lists ──────────────────────────

  import { StickyNote, ListTodo, Sparkles, ArrowRight } from '@lucide/svelte';

  import WidgetSpawnCard from './WidgetSpawnCard.svelte';
  import PinnedWidgetCard from './PinnedWidgetCard.svelte';
  import { notesStore } from '$lib/stores/notes.svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import { widgetsStore } from '$lib/stores/widgets.svelte';
  import { navStore } from '$lib/stores/nav.svelte';
  import * as widgetsApi from '$lib/ipc/widgets';

  $effect(() => {
    void widgetsStore.ensureSubscribed();
    void widgetsStore.refresh();
  });

  // Spawning a sticky from the hub creates a fresh note first, then pins it
  // immediately. The new note becomes the selection in the notes store so
  // the user can edit it inline from either the widget or the library.
  async function createStickyFromBlank() {
    const note = await notesStore.create();
    if (!note) return;
    await widgetsApi.pinNote(note.id);
  }

  async function createTodoFromBlank() {
    const list = await listsStore.create();
    if (!list) return;
    await widgetsApi.pinList(list.id);
  }

  // Resolve human-readable label + preview text for a pinned instance by
  // looking up its source row in the cached stores.
  function titleFor(w: widgetsApi.WidgetInstance): string {
    if (w.kind === 'sticky') {
      const n = notesStore.notes.find((x) => x.id === w.target_id);
      return n?.title.trim() || 'Untitled note';
    }
    const l = listsStore.lists.find((x) => x.id === w.target_id);
    return l?.title.trim() || 'Untitled list';
  }

  function previewFor(w: widgetsApi.WidgetInstance): string[] {
    if (w.kind === 'sticky') {
      const n = notesStore.notes.find((x) => x.id === w.target_id);
      if (!n) return [];
      return n.body.split(/\r?\n/).filter((l) => l.trim().length > 0);
    }
    // We don't pre-fetch tasks for unselected lists - keep preview simple.
    return [];
  }

  async function focusWidget(w: widgetsApi.WidgetInstance) {
    // Pinning an already-pinned target is idempotent in the backend (it
    // focuses the existing window), so we reuse that path.
    if (w.kind === 'sticky') await widgetsApi.pinNote(w.target_id);
    else await widgetsApi.pinList(w.target_id);
  }

  function recentNotes() {
    return notesStore.notes.slice(0, 4);
  }
  function recentLists() {
    return listsStore.lists.slice(0, 4);
  }

  function formatRelative(iso: string): string {
    const t = Date.parse(iso);
    if (Number.isNaN(t)) return '';
    const diff = (Date.now() - t) / 1000;
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86_400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86_400)}d ago`;
  }
</script>

<div class="flex h-full flex-col overflow-y-auto">
  <div class="mx-auto flex w-full max-w-5xl flex-col gap-8 p-8">
    <!-- Hero: brand greeting + tagline. The Sparkles + gradient text are the
         strongest "this app cares about its look" cue on first load. -->
    <header class="flex flex-col gap-2">
      <span
        class="inline-flex w-fit items-center gap-1.5 rounded-pill border border-border-subtle bg-surface-1 px-2.5 py-1 text-[11px] font-medium text-text-2 shadow-hd-xs"
      >
        <Sparkles size={12} class="text-accent" />
        Widget hub
      </span>
      <h1 class="text-3xl font-semibold leading-tight tracking-tight">
        <span class="text-text-1">Pin a piece of your day to your </span>
        <span class="gradient-text">desktop</span>
      </h1>
      <p class="max-w-xl text-sm text-text-2">
        Hoverdo is built around floating widgets. Spin up a sticky for a thought,
        a to-do list for a project — drag them anywhere, set them on top, dial
        the opacity. The main window is just a control tower.
      </p>
    </header>

    <!-- Spawn tiles. Each tile carries its own gradient identity so the two
         widget kinds feel like distinct "things you can pin", not generic
         buttons. -->
    <section class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <WidgetSpawnCard
        title="Sticky note"
        description="A paper-feel scratchpad. Title + body, auto-saved. Drop it next to whatever you're working on."
        accent="linear-gradient(135deg, #ff8a6b 0%, #ff6aa3 100%)"
        onspawn={createStickyFromBlank}
      >
        {#snippet icon()}
          <StickyNote size={20} />
        {/snippet}
      </WidgetSpawnCard>

      <WidgetSpawnCard
        title="To-do list"
        description="A floating checklist. Add tasks fast, tick them off, set reminders that fire as Windows notifications."
        accent="linear-gradient(135deg, #8c6bff 0%, #6b8cff 100%)"
        onspawn={createTodoFromBlank}
      >
        {#snippet icon()}
          <ListTodo size={20} />
        {/snippet}
      </WidgetSpawnCard>
    </section>

    <!-- Pinned gallery. Hidden entirely when nothing is pinned so the hub
         doesn't feel cluttered before the user has any widgets. -->
    {#if widgetsStore.widgets.length > 0}
      <section class="space-y-3">
        <div class="flex items-baseline justify-between">
          <h2 class="text-sm font-semibold text-text-1">
            On your desktop
            <span class="ml-1 text-text-3">· {widgetsStore.widgets.length}</span>
          </h2>
          <span class="text-[11px] text-text-3">Click to bring to front · ✕ to unpin</span>
        </div>
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {#each widgetsStore.widgets as w (w.id)}
            <PinnedWidgetCard
              widget={w}
              title={titleFor(w)}
              preview={previewFor(w)}
              onfocus={() => focusWidget(w)}
            />
          {/each}
        </div>
      </section>
    {/if}

    <!-- Recent surfaces. Two columns side-by-side; clicking jumps into the
         relevant library section with the item selected. -->
    <section class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <div class="glass-card p-4">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-semibold text-text-1">Recent notes</h2>
          <button
            type="button"
            class="inline-flex items-center gap-1 rounded-control px-1.5 py-0.5 text-[11px] text-text-2 hover:bg-surface-2 hover:text-text-1"
            onclick={() => navStore.set('notes')}
          >
            All
            <ArrowRight size={11} />
          </button>
        </div>
        {#if recentNotes().length === 0}
          <p class="py-6 text-center text-xs text-text-3">
            No notes yet — spin up a sticky above and your first note will land here.
          </p>
        {:else}
          <ul class="space-y-1">
            {#each recentNotes() as n (n.id)}
              <li>
                <button
                  type="button"
                  class="flex w-full items-center justify-between gap-3 rounded-control px-2 py-1.5 text-left hover:bg-surface-2"
                  onclick={() => {
                    notesStore.select(n.id);
                    navStore.set('notes');
                  }}
                >
                  <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm text-text-1">
                      {n.title.trim() || 'Untitled'}
                    </span>
                    <span class="block truncate text-[11px] text-text-3">
                      {n.body.trim().split(/\r?\n/)[0] || 'No content yet'}
                    </span>
                  </span>
                  <span class="shrink-0 text-[10px] text-text-3">
                    {formatRelative(n.updated_at)}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <div class="glass-card p-4">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-semibold text-text-1">Recent lists</h2>
          <button
            type="button"
            class="inline-flex items-center gap-1 rounded-control px-1.5 py-0.5 text-[11px] text-text-2 hover:bg-surface-2 hover:text-text-1"
            onclick={() => navStore.set('lists')}
          >
            All
            <ArrowRight size={11} />
          </button>
        </div>
        {#if recentLists().length === 0}
          <p class="py-6 text-center text-xs text-text-3">
            No lists yet — create a to-do widget above to start tracking tasks.
          </p>
        {:else}
          <ul class="space-y-1">
            {#each recentLists() as l (l.id)}
              <li>
                <button
                  type="button"
                  class="flex w-full items-center justify-between gap-3 rounded-control px-2 py-1.5 text-left hover:bg-surface-2"
                  onclick={() => {
                    void listsStore.select(l.id);
                    navStore.set('lists');
                  }}
                >
                  <span class="truncate text-sm text-text-1">
                    {l.title.trim() || 'Untitled list'}
                  </span>
                  <span class="shrink-0 text-[10px] text-text-3">
                    {formatRelative(l.updated_at)}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </section>
  </div>
</div>
