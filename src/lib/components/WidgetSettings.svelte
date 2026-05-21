<script lang="ts">
  // Shared settings strip used by sticky & todo widgets.
  // Lives outside the drag region so its controls are interactive.

  import { Pin, PinOff } from '@lucide/svelte';

  type Props = {
    opacity: number;
    alwaysOnTop: boolean;
    onOpacityChange: (value: number) => void;
    onAlwaysOnTopChange: (value: boolean) => void;
  };

  let {
    opacity,
    alwaysOnTop,
    onOpacityChange,
    onAlwaysOnTopChange
  }: Props = $props();

  // Debounce frontend → backend persistence so dragging the slider doesn't
  // hammer the DB. Visual opacity (parent) updates instantly.
  let persistTimer: number | null = null;
  function onSliderInput(e: Event) {
    const value = Number((e.currentTarget as HTMLInputElement).value);
    onOpacityChange(value);
    if (persistTimer !== null) window.clearTimeout(persistTimer);
    persistTimer = window.setTimeout(() => {
      onOpacityChange(value); // re-emit; the parent debounces persistence
    }, 200);
  }

  function pct(v: number): string {
    return `${Math.round(v * 100)}%`;
  }
</script>

<div
  class="flex shrink-0 items-center gap-2 border-t border-border-subtle px-2 py-1 text-[11px] text-text-3"
>
  <span class="shrink-0">Opacity</span>
  <input
    type="range"
    min="0.2"
    max="1"
    step="0.05"
    value={opacity}
    oninput={onSliderInput}
    class="hd-range flex-1"
    aria-label="Widget opacity"
  />
  <span class="w-9 shrink-0 text-right tabular-nums text-text-2">
    {pct(opacity)}
  </span>
  <button
    type="button"
    class="rounded-control p-1 transition-colors hover:bg-surface-2"
    class:text-accent={alwaysOnTop}
    class:text-text-2={!alwaysOnTop}
    aria-pressed={alwaysOnTop}
    aria-label={alwaysOnTop ? 'Disable always-on-top' : 'Enable always-on-top'}
    title={alwaysOnTop ? 'Always on top: on' : 'Always on top: off'}
    onclick={() => onAlwaysOnTopChange(!alwaysOnTop)}
  >
    {#if alwaysOnTop}
      <Pin size={12} class="fill-current" />
    {:else}
      <PinOff size={12} />
    {/if}
  </button>
</div>

<style>
  .hd-range {
    appearance: none;
    height: 4px;
    background: var(--hd-border);
    border-radius: 9999px;
  }
  .hd-range::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 9999px;
    background: var(--hd-accent);
    border: 2px solid var(--hd-surface-1);
    box-shadow: var(--hd-shadow-sm);
    cursor: pointer;
  }
  .hd-range::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 9999px;
    background: var(--hd-accent);
    border: 2px solid var(--hd-surface-1);
    cursor: pointer;
  }
</style>
