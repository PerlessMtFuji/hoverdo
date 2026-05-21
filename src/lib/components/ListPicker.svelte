<script lang="ts">
  import { Plus, ListChecks } from '@lucide/svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import Button from './Button.svelte';

  function relativeTime(iso: string): string {
    const then = new Date(iso).getTime();
    const diff = Math.max(0, (Date.now() - then) / 1000);
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86_400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86_400)}d`;
  }
</script>

<div class="flex h-full w-72 shrink-0 flex-col border-r border-border-subtle bg-surface-1/40">
  <div
    class="flex h-12 shrink-0 items-center justify-between border-b border-border-subtle px-3"
  >
    <span class="text-sm font-medium text-text-1">To-do lists</span>
    <Button
      variant="primary"
      size="sm"
      onclick={() => listsStore.create()}
      aria-label="New list"
    >
      <Plus size={14} />
      New
    </Button>
  </div>

  <ul class="flex-1 overflow-y-auto p-2">
    {#if listsStore.loading && listsStore.lists.length === 0}
      <li class="px-2 py-6 text-center text-sm text-text-3">Loading…</li>
    {:else if listsStore.lists.length === 0}
      <li class="flex flex-col items-center gap-2 px-2 py-10 text-text-3">
        <ListChecks size={28} class="opacity-50" />
        <p class="text-sm">No lists yet</p>
        <p class="text-xs">Hit "New" to start one.</p>
      </li>
    {:else}
      {#each listsStore.lists as list (list.id)}
        <li>
          <button
            type="button"
            class="flex w-full items-baseline justify-between gap-2 rounded-control px-3 py-2 text-left transition-colors hover:bg-surface-2"
            class:bg-surface-2={listsStore.selectedId === list.id}
            onclick={() => listsStore.select(list.id)}
          >
            <span class="truncate text-sm font-medium text-text-1">
              {list.title.trim() || 'Untitled list'}
            </span>
            <span class="shrink-0 text-[11px] text-text-3">
              {relativeTime(list.updated_at)}
            </span>
          </button>
        </li>
      {/each}
    {/if}
  </ul>

  {#if listsStore.error}
    <div
      class="border-t border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger"
    >
      {listsStore.error}
    </div>
  {/if}
</div>
