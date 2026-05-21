<script lang="ts">
  // Global search: full-text across notes / lists / tasks. Ctrl/Cmd+F
  // focuses the input from anywhere; Esc clears and blurs. Selecting a
  // result hops to the right section and selects the underlying item.

  import { Search, FileText, ListChecks, ClipboardCheck } from '@lucide/svelte';
  import { fly } from 'svelte/transition';
  import { search as fts, type SearchHit } from '$lib/ipc/search';
  import { notesStore } from '$lib/stores/notes.svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import { navStore } from '$lib/stores/nav.svelte';

  let query = $state('');
  let hits = $state<SearchHit[]>([]);
  let open = $state(false);
  let inputEl: HTMLInputElement | null = $state(null);
  let lastQuery = '';
  let runToken = 0;

  $effect(() => {
    const q = query.trim();
    if (q === lastQuery) return;
    lastQuery = q;
    if (q.length === 0) {
      hits = [];
      return;
    }
    const myToken = ++runToken;
    void (async () => {
      try {
        const result = await fts(q);
        if (myToken === runToken) hits = result;
      } catch (e) {
        if (myToken === runToken) hits = [];
        console.warn('search failed', e);
      }
    })();
  });

  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      inputEl?.focus();
      inputEl?.select();
      open = true;
    } else if (e.key === 'Escape' && document.activeElement === inputEl) {
      query = '';
      hits = [];
      open = false;
      inputEl?.blur();
    }
  }

  async function pick(hit: SearchHit) {
    if (hit.kind === 'note') {
      navStore.set('notes');
      notesStore.select(hit.id);
    } else if (hit.kind === 'list') {
      navStore.set('lists');
      await listsStore.select(hit.id);
    } else if (hit.kind === 'task') {
      navStore.set('lists');
      if (hit.list_id) await listsStore.select(hit.list_id);
    }
    query = '';
    hits = [];
    open = false;
    inputEl?.blur();
  }

  // Snippets come with U+0001/U+0002 control characters around the matched
  // span. We don't trust backend HTML; this parser converts the markers
  // into structured spans so Svelte renders them safely.
  function renderSnippet(snippet: string): Array<{ text: string; match: boolean }> {
    const parts: Array<{ text: string; match: boolean }> = [];
    let inMatch = false;
    let buf = '';
    for (const ch of snippet) {
      if (ch === '\x01') {
        if (buf) parts.push({ text: buf, match: inMatch });
        buf = '';
        inMatch = true;
      } else if (ch === '\x02') {
        if (buf) parts.push({ text: buf, match: inMatch });
        buf = '';
        inMatch = false;
      } else {
        buf += ch;
      }
    }
    if (buf) parts.push({ text: buf, match: inMatch });
    return parts;
  }

  function iconFor(kind: string) {
    return kind === 'note' ? FileText : kind === 'list' ? ListChecks : ClipboardCheck;
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="relative" data-no-drag>
  <div class="relative">
    <Search
      size={13}
      class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-text-3"
    />
    <input
      bind:this={inputEl}
      bind:value={query}
      type="text"
      placeholder="Search…  Ctrl+F"
      class="h-7 w-56 rounded-control border border-border-subtle bg-surface-1/60 pl-7 pr-2 text-xs text-text-1 placeholder:text-text-3 focus:bg-surface-1 focus:outline-none focus:ring-1 focus:ring-accent"
      onfocus={() => (open = true)}
      onblur={() => {
        // Defer so click on a result lands before we close.
        window.setTimeout(() => (open = false), 120);
      }}
    />
  </div>

  {#if open && (hits.length > 0 || query.trim().length > 0)}
    <div
      transition:fly={{ y: -4, duration: 140 }}
      class="absolute right-0 top-full z-30 mt-1 max-h-96 w-80 overflow-y-auto rounded-control border border-border-subtle bg-surface-1/95 p-1 shadow-hd-lg backdrop-blur"
    >
      {#if hits.length === 0}
        <p class="px-3 py-3 text-center text-xs text-text-3">No matches</p>
      {:else}
        {#each hits as hit (hit.kind + hit.id)}
          {@const Icon = iconFor(hit.kind)}
          <button
            type="button"
            class="flex w-full items-start gap-2 rounded-control px-2 py-1.5 text-left hover:bg-surface-2"
            onmousedown={(e) => {
              e.preventDefault();
              void pick(hit);
            }}
          >
            <Icon size={13} class="mt-0.5 shrink-0 text-text-3" />
            <span class="flex min-w-0 flex-1 flex-col gap-0.5">
              <span class="truncate text-xs font-medium text-text-1">
                {hit.title.trim() || 'Untitled'}
              </span>
              <span class="line-clamp-2 text-[11px] leading-snug text-text-2">
                {#each renderSnippet(hit.snippet) as part, i (i)}
                  {#if part.match}
                    <mark class="rounded bg-accent/20 px-0.5 text-text-1">
                      {part.text}
                    </mark>
                  {:else}
                    <span>{part.text}</span>
                  {/if}
                {/each}
              </span>
            </span>
            <span
              class="shrink-0 self-center rounded bg-surface-2 px-1 py-0.5 text-[9px] uppercase tracking-wider text-text-3"
            >
              {hit.kind}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>
