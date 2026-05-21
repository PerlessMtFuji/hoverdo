<script lang="ts">
  import { StickyNote, ListChecks, Tag } from '@lucide/svelte';

  // Static nav placeholder until lists / tags ship in their own commits.
  // The "Notes" item is selected; the others are inert and just hint at
  // what's coming next.
</script>

<aside
  class="flex w-44 shrink-0 flex-col gap-0.5 border-r border-border-subtle bg-surface-1/30 p-3 text-sm text-text-2"
>
  <p class="px-2 pb-2 text-[10px] uppercase tracking-wider text-text-3">
    Library
  </p>

  <button
    type="button"
    class="flex items-center gap-2 rounded-control bg-surface-2 px-2 py-1.5 text-left text-text-1"
  >
    <StickyNote size={14} />
    Notes
  </button>

  <button
    type="button"
    disabled
    title="Coming next"
    class="flex items-center gap-2 rounded-control px-2 py-1.5 text-left opacity-50"
  >
    <ListChecks size={14} />
    To-do lists
  </button>

  <button
    type="button"
    disabled
    title="Coming next"
    class="flex items-center gap-2 rounded-control px-2 py-1.5 text-left opacity-50"
  >
    <Tag size={14} />
    Tags
  </button>
</aside>
