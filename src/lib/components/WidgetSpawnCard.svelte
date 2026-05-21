<script lang="ts">
  // Big hero tile in the home hub - "create a new widget of this kind".
  // Each tile is a self-contained gradient card with an icon, label and a
  // single-shot async action. The tile takes care of disabling itself while
  // the spawn is in flight and surfacing failures inline (no separate
  // toast layer in MVP).

  import type { Snippet } from 'svelte';
  import { Plus } from '@lucide/svelte';

  type Props = {
    title: string;
    description: string;
    accent: string; // CSS gradient string
    onspawn: () => Promise<void> | void;
    icon: Snippet;
  };
  let { title, description, accent, onspawn, icon }: Props = $props();

  let busy = $state(false);
  let error = $state<string | null>(null);

  async function activate() {
    if (busy) return;
    busy = true;
    error = null;
    try {
      await onspawn();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<button
  type="button"
  onclick={activate}
  disabled={busy}
  class="group relative flex h-44 w-full flex-col justify-between overflow-hidden rounded-card border border-border-subtle bg-surface-1 p-5 text-left shadow-hd-md transition-all duration-200 hover:-translate-y-0.5 hover:shadow-hd-lg disabled:cursor-progress disabled:opacity-70"
>
  <!-- Painted gradient blob in the corner: provides the "this widget kind has
       its own personality" cue without bleeding into the legibility of text. -->
  <span
    aria-hidden="true"
    class="pointer-events-none absolute -right-12 -top-12 h-44 w-44 rounded-full opacity-80 blur-2xl transition-opacity duration-300 group-hover:opacity-100"
    style="background: {accent};"
  ></span>
  <span
    aria-hidden="true"
    class="pointer-events-none absolute inset-x-0 -bottom-12 h-24 opacity-60 blur-2xl"
    style="background: {accent};"
  ></span>

  <div class="relative flex items-start justify-between gap-3">
    <span
      class="flex h-11 w-11 items-center justify-center rounded-control text-white shadow-hd-sm"
      style="background: {accent};"
    >
      {@render icon()}
    </span>
    <span
      class="rounded-pill border border-border-subtle bg-surface-2 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider text-text-3 backdrop-blur"
    >
      New
    </span>
  </div>

  <div class="relative space-y-1">
    <h3 class="text-base font-semibold text-text-1">{title}</h3>
    <p class="text-xs leading-relaxed text-text-2">{description}</p>
    <div
      class="flex items-center gap-1 pt-2 text-xs font-medium text-accent transition-transform duration-200 group-hover:translate-x-0.5"
    >
      <Plus size={13} />
      Create
    </div>
  </div>

  {#if error}
    <span
      class="absolute inset-x-3 bottom-2 truncate rounded-control bg-danger/10 px-2 py-1 text-[10px] text-danger"
    >
      {error}
    </span>
  {/if}
</button>
