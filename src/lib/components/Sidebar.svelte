<script lang="ts">
  // Library sidebar. Visual goals:
  //  • Home = widget hub, given top-billing as the brand entry point.
  //  • Notes + Lists below as library surfaces.
  //  • Active item gets a gradient pill so navigation feels "alive" and the
  //    interface doesn't reduce to grey-on-grey hover states.

  import { LayoutDashboard, FileText, ListChecks, Tag } from '@lucide/svelte';
  import { navStore, type Section } from '$lib/stores/nav.svelte';
  import { notesStore } from '$lib/stores/notes.svelte';
  import { listsStore } from '$lib/stores/lists.svelte';

  type NavItem = {
    id: Section;
    label: string;
    icon: typeof LayoutDashboard;
    count: () => number;
  };

  const items: NavItem[] = [
    {
      id: 'home',
      label: 'Hub',
      icon: LayoutDashboard,
      count: () => 0
    },
    {
      id: 'notes',
      label: 'Notes',
      icon: FileText,
      count: () => notesStore.notes.length
    },
    {
      id: 'lists',
      label: 'To-do lists',
      icon: ListChecks,
      count: () => listsStore.lists.length
    }
  ];
</script>

<aside
  class="flex w-48 shrink-0 flex-col gap-1 border-r border-border-subtle px-3 py-4 text-sm text-text-2"
>
  <div class="px-2 pb-3">
    <p class="text-[10px] font-semibold uppercase tracking-[0.15em] text-text-3">
      Workspace
    </p>
  </div>

  {#each items as item (item.id)}
    {@const Icon = item.icon}
    {@const active = navStore.section === item.id}
    {@const n = item.count()}
    <button
      type="button"
      onclick={() => navStore.set(item.id)}
      class="group relative flex items-center gap-2.5 rounded-control px-2.5 py-2 text-left text-sm transition-all duration-150 ease-out"
      class:bg-surface-2={active}
      class:text-text-1={active}
      class:shadow-hd-xs={active}
      class:hover:bg-surface-1={!active}
      class:hover:text-text-1={!active}
    >
      {#if active}
        <span
          aria-hidden="true"
          class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-pill"
          style="background: var(--hd-gradient-brand);"
        ></span>
      {/if}
      <Icon size={15} class={active ? 'text-accent' : 'text-text-3'} />
      <span class="flex-1 truncate">{item.label}</span>
      {#if n > 0}
        <span
          class="rounded-pill px-1.5 py-0.5 text-[10px] font-medium tabular-nums"
          class:bg-accent-soft={active}
          class:text-accent={active}
          class:bg-surface-sunken={!active}
          class:text-text-3={!active}
        >
          {n}
        </span>
      {/if}
    </button>
  {/each}

  <div class="mt-4 px-2 pb-2">
    <p class="text-[10px] font-semibold uppercase tracking-[0.15em] text-text-3">
      Soon
    </p>
  </div>
  <button
    type="button"
    disabled
    title="Tags - coming next"
    class="flex items-center gap-2.5 rounded-control px-2.5 py-2 text-left text-sm opacity-40"
  >
    <Tag size={15} />
    Tags
  </button>
</aside>
