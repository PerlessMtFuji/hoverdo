<script lang="ts">
  // Hoverdo - main window
  // MVP scaffolding: real CRUD wires up in the next commit once the DB layer lands.
  // For now this proves theming, Mica transparency and Svelte 5 runes are alive.

  import { theme, toggleTheme } from '$lib/theme/theme.svelte';
  import { Sun, Moon } from '@lucide/svelte';

  let counter = $state(0);
</script>

<div class="flex h-full flex-col">
  <!-- Title bar (drag region, borderless window) -->
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
    <!-- Sidebar placeholder -->
    <aside
      class="w-56 shrink-0 border-r border-border-subtle bg-surface-1/40 p-3 text-sm text-text-2"
    >
      <p class="px-2 pb-2 text-xs uppercase tracking-wider text-text-3">
        Library
      </p>
      <nav class="flex flex-col gap-0.5">
        <button
          type="button"
          class="rounded-control bg-surface-2 px-2 py-1.5 text-left text-text-1"
        >
          All notes
        </button>
        <button
          type="button"
          class="rounded-control px-2 py-1.5 text-left hover:bg-surface-2"
        >
          To-do lists
        </button>
        <button
          type="button"
          class="rounded-control px-2 py-1.5 text-left hover:bg-surface-2"
        >
          Tags
        </button>
      </nav>
    </aside>

    <!-- Content placeholder -->
    <section class="flex-1 overflow-auto p-6">
      <h1 class="text-2xl font-semibold tracking-tight text-text-1">
        Welcome to Hoverdo
      </h1>
      <p class="mt-2 max-w-prose text-text-2">
        Floating-widget-first notes &amp; todos. This is the main window
        scaffold - sticky-note and to-do list widgets, full CRUD, tags, search
        and reminders arrive in upcoming commits.
      </p>

      <div class="mt-6 flex items-center gap-3">
        <button
          type="button"
          class="rounded-control bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg shadow-hd-sm transition-transform active:scale-[0.98]"
          onclick={() => counter++}
        >
          Smoke-test ({counter})
        </button>
        <span class="text-sm text-text-3">
          Theme: <span class="text-text-2">{theme.resolved}</span>
          {#if theme.source === 'system'}<span class="text-text-3">
              (system)</span
            >{/if}
        </span>
      </div>
    </section>
  </main>
</div>
