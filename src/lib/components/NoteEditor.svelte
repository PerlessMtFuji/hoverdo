<script lang="ts">
  import { notesStore } from '$lib/stores/notes.svelte';
  import { Trash2, FilePenLine } from '@lucide/svelte';
  import Button from './Button.svelte';

  // Debounced auto-save: edits accumulate locally, then flush 500ms after
  // the last keystroke. We track which note we last loaded so switching
  // selections never overwrites the new note with the old draft.
  const SAVE_DEBOUNCE_MS = 500;

  let draftTitle = $state('');
  let draftBody = $state('');
  let loadedId: string | null = $state(null);
  let saveTimer: number | null = null;
  let saving = $state(false);

  // When the selection changes, sync the editor draft to the new note's
  // content. Pending saves for the previous note are flushed first.
  $effect(() => {
    const sel = notesStore.selected;
    if (sel?.id !== loadedId) {
      flushNow();
      loadedId = sel?.id ?? null;
      draftTitle = sel?.title ?? '';
      draftBody = sel?.body ?? '';
    }
  });

  function schedule() {
    if (!loadedId) return;
    if (saveTimer !== null) window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(flushNow, SAVE_DEBOUNCE_MS);
  }

  async function flushNow() {
    if (saveTimer !== null) {
      window.clearTimeout(saveTimer);
      saveTimer = null;
    }
    const id = loadedId;
    if (!id) return;
    const current = notesStore.notes.find((n) => n.id === id);
    if (!current) return;
    if (current.title === draftTitle && current.body === draftBody) return;
    saving = true;
    try {
      await notesStore.update(id, { title: draftTitle, body: draftBody });
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    const id = loadedId;
    if (!id) return;
    flushNow();
    await notesStore.remove(id);
  }
</script>

<section class="flex h-full flex-1 flex-col">
  {#if !notesStore.selected}
    <div class="m-auto flex flex-col items-center gap-2 text-text-3">
      <FilePenLine size={32} class="opacity-50" />
      <p class="text-sm">Select a note or create a new one.</p>
    </div>
  {:else}
    <header
      class="flex h-12 shrink-0 items-center justify-between border-b border-border-subtle px-4"
    >
      <span class="text-xs text-text-3">
        {#if saving}Saving...{:else}Edited
          {new Date(notesStore.selected.updated_at).toLocaleString()}{/if}
      </span>
      <Button variant="danger" size="sm" onclick={handleDelete}>
        <Trash2 size={14} />
        Delete
      </Button>
    </header>

    <div class="flex flex-1 flex-col overflow-hidden px-6 py-4">
      <input
        type="text"
        bind:value={draftTitle}
        oninput={schedule}
        onblur={flushNow}
        placeholder="Title"
        class="w-full border-0 bg-transparent text-xl font-semibold tracking-tight text-text-1 placeholder:text-text-3 focus:outline-none"
      />
      <textarea
        bind:value={draftBody}
        oninput={schedule}
        onblur={flushNow}
        placeholder="Start writing..."
        class="mt-3 flex-1 resize-none border-0 bg-transparent text-sm leading-relaxed text-text-1 placeholder:text-text-3 focus:outline-none"
      ></textarea>
    </div>
  {/if}
</section>
