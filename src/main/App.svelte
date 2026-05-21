<script lang="ts">
  import { theme, toggleTheme } from '$lib/theme/theme.svelte';
  import { notesStore } from '$lib/stores/notes.svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import { navStore } from '$lib/stores/nav.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import HomeHub from '$lib/components/HomeHub.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import ListPicker from '$lib/components/ListPicker.svelte';
  import TaskBoard from '$lib/components/TaskBoard.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import { Sun, Moon } from '@lucide/svelte';

  $effect(() => {
    void notesStore.refresh();
    void notesStore.ensureSubscribed();
    void listsStore.refresh();
    void listsStore.ensureSubscribed();
  });

  // Ctrl/Cmd+N → create in the active library section. From the hub we
  // default to "new note" since that's the most common scratch action.
  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      if (navStore.section === 'lists') void listsStore.create();
      else void notesStore.create();
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="flex h-full flex-col">
  <!-- Top bar. Drag region with the wordmark on the left (gradient!) and the
       search + theme toggle on the right. Keeping height at 44px so the
       chrome stays light. -->
  <header
    class="drag-region flex h-11 shrink-0 items-center gap-3 border-b border-border-subtle px-4"
  >
    <span class="flex items-center gap-2">
      <span
        aria-hidden="true"
        class="block h-4 w-4 rounded-pill shadow-hd-sm"
        style="background: var(--hd-gradient-brand);"
      ></span>
      <span class="gradient-text text-sm font-semibold tracking-tight">
        Hoverdo
      </span>
    </span>
    <div class="flex-1"></div>
    <SearchBar />
    <button
      type="button"
      class="rounded-control p-1.5 text-text-2 transition-colors hover:bg-surface-2 hover:text-text-1"
      aria-label="Toggle theme"
      data-no-drag
      onclick={toggleTheme}
    >
      {#if theme.resolved === 'dark'}
        <Sun size={15} />
      {:else}
        <Moon size={15} />
      {/if}
    </button>
  </header>

  <main class="flex flex-1 overflow-hidden">
    <Sidebar />
    {#if navStore.section === 'home'}
      <HomeHub />
    {:else if navStore.section === 'notes'}
      <NoteList />
      <NoteEditor />
    {:else}
      <ListPicker />
      <TaskBoard />
    {/if}
  </main>
</div>
