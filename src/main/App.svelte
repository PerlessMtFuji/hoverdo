<script lang="ts">
  import { theme, toggleTheme } from '$lib/theme/theme.svelte';
  import { notesStore } from '$lib/stores/notes.svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import { navStore } from '$lib/stores/nav.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import ListPicker from '$lib/components/ListPicker.svelte';
  import TaskBoard from '$lib/components/TaskBoard.svelte';
  import { Sun, Moon } from '@lucide/svelte';

  $effect(() => {
    void notesStore.refresh();
    void notesStore.ensureSubscribed();
    void listsStore.refresh();
    void listsStore.ensureSubscribed();
  });

  // Ctrl/Cmd+N → new item in the active section.
  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      if (navStore.section === 'notes') void notesStore.create();
      else void listsStore.create();
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
    {#if navStore.section === 'notes'}
      <NoteList />
      <NoteEditor />
    {:else}
      <ListPicker />
      <TaskBoard />
    {/if}
  </main>
</div>
