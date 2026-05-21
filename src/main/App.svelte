<script lang="ts">
  import { theme, toggleTheme } from '$lib/theme/theme.svelte';
  import { notesStore } from '$lib/stores/notes.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import { Sun, Moon } from '@lucide/svelte';

  // Initial load + cross-window subscription.
  $effect(() => {
    void notesStore.refresh();
    void notesStore.ensureSubscribed();
  });

  // Ctrl+N → new note (matches the plan's keyboard-shortcut promise).
  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      void notesStore.create();
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="flex h-full flex-col">
  <header
    class="drag-region flex h-10 shrink-0 items-center justify-between border-b border-border-subtle px-4 text-text-2"
  >
    <span class="text-sm font-medium tracking-tight text-text-1">Hoverdo</span>
    <button
      type="button"
      class="rounded-control p-1 text-text-2 transition-colors hover:bg-surface-2 hover:text-text-1"
      aria-label="Toggle theme"
      data-no-drag
      onclick={toggleTheme}
    >
      {#if theme.resolved === 'dark'}
        <Sun size={16} />
      {:else}
        <Moon size={16} />
      {/if}
    </button>
  </header>

  <main class="flex flex-1 overflow-hidden">
    <Sidebar />
    <NoteList />
    <NoteEditor />
  </main>
</div>
