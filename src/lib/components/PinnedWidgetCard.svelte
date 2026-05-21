<script lang="ts">
  // Compact thumbnail of an already-pinned widget. Doubles as both a
  // jump-back-to-it affordance (focuses the window) and as a way to unpin
  // when the user no longer wants the widget on the desktop.
  //
  // Title + preview come from the source row (note body, list tasks) so the
  // user recognises the widget at a glance even when several are pinned.

  import { ExternalLink, X, StickyNote, ListTodo } from '@lucide/svelte';
  import * as widgetsApi from '$lib/ipc/widgets';

  type Props = {
    widget: widgetsApi.WidgetInstance;
    title: string;
    preview: string[]; // small preview lines (body or tasks)
    onfocus: () => void;
  };
  let { widget, title, preview, onfocus }: Props = $props();

  let unpinning = $state(false);

  // Paper palette indexed by widget id so colour is stable per widget but
  // varies across the gallery - little visual variety even before the user
  // sets a custom colour.
  const palette = [
    'var(--hd-paper-butter)',
    'var(--hd-paper-peach)',
    'var(--hd-paper-rose)',
    'var(--hd-paper-lilac)',
    'var(--hd-paper-mint)',
    'var(--hd-paper-sky)',
    'var(--hd-paper-sand)'
  ];
  function bgFor(id: string): string {
    let hash = 0;
    for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) | 0;
    return palette[Math.abs(hash) % palette.length];
  }
  // Derived so reactive prop changes flow through (in practice the widget id
  // is stable for a card's lifetime, but $derived keeps the analyser happy).
  const accent = $derived(bgFor(widget.id));

  async function unpin(e: MouseEvent) {
    e.stopPropagation();
    if (unpinning) return;
    unpinning = true;
    try {
      await widgetsApi.unpinWidget(widget.id);
    } finally {
      unpinning = false;
    }
  }
</script>

<button
  type="button"
  onclick={onfocus}
  class="group relative flex h-32 w-full flex-col overflow-hidden rounded-card border border-border-subtle bg-surface-1 p-3 text-left shadow-hd-sm transition-all duration-150 hover:-translate-y-0.5 hover:shadow-hd-md"
>
  <!-- Colored ribbon along the top edge: signature of the widget instance. -->
  <span
    aria-hidden="true"
    class="absolute inset-x-0 top-0 h-1.5"
    style="background: linear-gradient(90deg, {accent}, color-mix(in srgb, {accent} 70%, transparent));"
  ></span>

  <div class="mt-1 flex items-start gap-2">
    <span
      class="flex h-6 w-6 shrink-0 items-center justify-center rounded-pill text-text-1"
      style="background: color-mix(in srgb, {accent} 80%, transparent);"
    >
      {#if widget.kind === 'sticky'}
        <StickyNote size={12} />
      {:else}
        <ListTodo size={12} />
      {/if}
    </span>
    <span class="min-w-0 flex-1">
      <span class="block truncate text-sm font-medium text-text-1">
        {title || 'Untitled'}
      </span>
      <span class="text-[10px] uppercase tracking-wider text-text-3">
        {widget.kind === 'sticky' ? 'Sticky' : 'To-do'}
      </span>
    </span>
    <span
      class="flex shrink-0 gap-0.5 opacity-0 transition-opacity group-hover:opacity-100"
    >
      <span
        class="rounded-control p-1 text-text-3 hover:bg-surface-2 hover:text-accent"
        title="Open widget window"
      >
        <ExternalLink size={12} />
      </span>
      <span
        role="button"
        tabindex="0"
        aria-label="Unpin widget"
        title="Unpin"
        onclick={unpin}
        onkeydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            void unpin(e as unknown as MouseEvent);
          }
        }}
        class="rounded-control p-1 text-text-3 hover:bg-danger/10 hover:text-danger"
      >
        <X size={12} />
      </span>
    </span>
  </div>

  <div class="mt-2 flex-1 space-y-1 overflow-hidden text-[11px] leading-snug text-text-2">
    {#if preview.length === 0}
      <span class="italic text-text-3">No content yet</span>
    {:else}
      {#each preview.slice(0, 3) as line, i (i)}
        <span class="block truncate">{line}</span>
      {/each}
    {/if}
  </div>
</button>
