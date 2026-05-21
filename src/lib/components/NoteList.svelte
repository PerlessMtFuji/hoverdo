<script lang="ts">
  import type { Note } from '$lib/ipc/types';
  import { notesStore } from '$lib/stores/notes.svelte';
  import * as widgetsApi from '$lib/ipc/widgets';
  import { Plus, FileText, Pin } from '@lucide/svelte';
  import Button from './Button.svelte';

  async function pin(noteId: string, e: MouseEvent) {
    e.stopPropagation();
    try {
      await widgetsApi.pinNote(noteId);
    } catch (err) {
      console.warn('pin failed', err);
    }
  }

  function previewText(note: Note): string {
    const trimmed = note.body.trim();
    return trimmed.length > 0 ? trimmed : 'No content yet';
  }

  function previewTitle(note: Note): string {
    const t = note.title.trim();
    return t.length > 0 ? t : 'Untitled';
  }

  function relativeTime(iso: string): string {
    const then = new Date(iso).getTime();
    const diffSec = Math.max(0, (Date.now() - then) / 1000);
    if (diffSec < 60) return 'just now';
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m`;
    if (diffSec < 86_400) return `${Math.floor(diffSec / 3600)}h`;
    return `${Math.floor(diffSec / 86_400)}d`;
  }
</script>

<div class="flex h-full w-72 shrink-0 flex-col border-r border-border-subtle bg-surface-1/40">
  <div
    class="flex h-12 shrink-0 items-center justify-between border-b border-border-subtle px-3"
  >
    <span class="text-sm font-medium text-text-1">Notes</span>
    <Button
      variant="primary"
      size="sm"
      onclick={() => notesStore.create()}
      aria-label="New note"
    >
      <Plus size={14} />
      New
    </Button>
  </div>

  <ul class="flex-1 overflow-y-auto p-2">
    {#if notesStore.loading && notesStore.notes.length === 0}
      <li class="px-2 py-6 text-center text-sm text-text-3">Loading...</li>
    {:else if notesStore.notes.length === 0}
      <li class="flex flex-col items-center gap-2 px-2 py-10 text-text-3">
        <FileText size={28} class="opacity-50" />
        <p class="text-sm">No notes yet</p>
        <p class="text-xs">Hit "New" to create your first one.</p>
      </li>
    {:else}
      {#each notesStore.notes as note (note.id)}
        <li>
          <div
            class="group relative rounded-control transition-colors hover:bg-surface-2"
            class:bg-surface-2={notesStore.selectedId === note.id}
          >
            <button
              type="button"
              class="flex w-full flex-col gap-1 rounded-control px-3 py-2 text-left"
              onclick={() => notesStore.select(note.id)}
            >
              <div class="flex items-baseline justify-between gap-2 pr-6">
                <span class="truncate text-sm font-medium text-text-1">
                  {previewTitle(note)}
                </span>
                <span class="shrink-0 text-[11px] text-text-3">
                  {relativeTime(note.updated_at)}
                </span>
              </div>
              <span class="line-clamp-2 text-xs leading-snug text-text-2">
                {previewText(note)}
              </span>
            </button>
            <button
              type="button"
              class="absolute right-1.5 top-1.5 rounded-control p-1 text-text-3 opacity-0 transition-opacity hover:bg-surface-1 hover:text-accent group-hover:opacity-100"
              aria-label="Pin as sticky"
              title="Pin as sticky note"
              onclick={(e) => pin(note.id, e)}
            >
              <Pin size={13} />
            </button>
          </div>
        </li>
      {/each}
    {/if}
  </ul>

  {#if notesStore.error}
    <div
      class="border-t border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger"
    >
      {notesStore.error}
    </div>
  {/if}
</div>
