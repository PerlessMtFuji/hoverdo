<script lang="ts">
  import { Trash2, Plus, ClipboardList } from '@lucide/svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import Button from './Button.svelte';
  import ReminderControl from './ReminderControl.svelte';

  let titleDraft = $state('');
  let renamingTitle = $state('');
  let lastSelectedId: string | null = null;

  $effect(() => {
    const sel = listsStore.selected;
    if (sel?.id !== lastSelectedId) {
      lastSelectedId = sel?.id ?? null;
      renamingTitle = sel?.title ?? '';
      titleDraft = '';
    }
  });

  async function addTask(e: SubmitEvent) {
    e.preventDefault();
    if (!titleDraft.trim()) return;
    await listsStore.addTask(titleDraft);
    titleDraft = '';
  }

  async function commitRename() {
    const sel = listsStore.selected;
    if (!sel) return;
    if (renamingTitle === sel.title) return;
    await listsStore.rename(sel.id, renamingTitle);
  }

  async function removeList() {
    const sel = listsStore.selected;
    if (!sel) return;
    await listsStore.remove(sel.id);
  }
</script>

<section class="flex h-full flex-1 flex-col">
  {#if !listsStore.selected}
    <div class="m-auto flex flex-col items-center gap-2 text-text-3">
      <ClipboardList size={32} class="opacity-50" />
      <p class="text-sm">Select a list or create a new one.</p>
    </div>
  {:else}
    <header
      class="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border-subtle px-4"
    >
      <input
        type="text"
        bind:value={renamingTitle}
        onblur={commitRename}
        onkeydown={(e) => {
          if (e.key === 'Enter') (e.currentTarget as HTMLInputElement).blur();
        }}
        class="flex-1 border-0 bg-transparent text-base font-semibold tracking-tight text-text-1 placeholder:text-text-3 focus:outline-none"
        placeholder="Untitled list"
      />
      <Button variant="danger" size="sm" onclick={removeList}>
        <Trash2 size={14} />
        Delete
      </Button>
    </header>

    <div class="flex flex-1 flex-col overflow-hidden">
      <ul class="flex-1 overflow-y-auto px-4 py-3">
        {#if listsStore.loadingTasks && listsStore.tasks.length === 0}
          <li class="px-2 py-6 text-center text-sm text-text-3">Loading…</li>
        {:else if listsStore.tasks.length === 0}
          <li class="px-2 py-6 text-center text-sm text-text-3">
            No tasks yet. Add the first one below.
          </li>
        {:else}
          {#each listsStore.tasks as task (task.id)}
            <li
              class="group flex items-center gap-2 rounded-control px-2 py-1.5 hover:bg-surface-2"
            >
              <input
                type="checkbox"
                checked={task.done}
                onchange={(e) =>
                  listsStore.toggleTask(
                    task.id,
                    (e.currentTarget as HTMLInputElement).checked
                  )}
                class="size-4 accent-accent"
              />
              <span
                class="flex-1 text-sm"
                class:line-through={task.done}
                class:text-text-3={task.done}
              >
                {task.title}
              </span>
              <ReminderControl taskId={task.id} reminder={listsStore.reminders[task.id]} />
              <button
                type="button"
                class="rounded-control p-1 text-text-3 opacity-0 transition-opacity hover:bg-danger/10 hover:text-danger group-hover:opacity-100"
                aria-label="Delete task"
                onclick={() => listsStore.removeTask(task.id)}
              >
                <Trash2 size={12} />
              </button>
            </li>
          {/each}
        {/if}
      </ul>

      <form
        onsubmit={addTask}
        class="flex shrink-0 items-center gap-2 border-t border-border-subtle px-4 py-3"
      >
        <input
          type="text"
          bind:value={titleDraft}
          placeholder="Add a task…"
          class="flex-1 rounded-control bg-surface-1 px-3 py-1.5 text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:ring-1 focus:ring-accent"
        />
        <Button variant="primary" size="sm" type="submit">
          <Plus size={14} />
          Add
        </Button>
      </form>
    </div>
  {/if}
</section>
