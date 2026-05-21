<script lang="ts">
  // Shared settings strip used by sticky & todo widgets.
  //
  // The `tone` prop controls whether the strip styles itself against the
  // paper backdrop of a sticky note (semi-transparent black) or against the
  // glassy backdrop of a todo widget (theme tokens). This keeps the API
  // surface tiny while still letting each widget feel native to its body.

  import { Pin, PinOff } from '@lucide/svelte';

  type Props = {
    opacity: number;
    alwaysOnTop: boolean;
    onOpacityChange: (value: number) => void;
    onAlwaysOnTopChange: (value: boolean) => void;
    tone?: 'paper' | 'glass';
  };

  let {
    opacity,
    alwaysOnTop,
    onOpacityChange,
    onAlwaysOnTopChange,
    tone = 'glass'
  }: Props = $props();

  function onSliderInput(e: Event) {
    const value = Number((e.currentTarget as HTMLInputElement).value);
    onOpacityChange(value);
  }

  function pct(v: number): string {
    return `${Math.round(v * 100)}%`;
  }
</script>

<div
  class="hd-settings flex shrink-0 items-center gap-2 rounded-control px-2 py-1.5 text-[11px]"
  data-tone={tone}
>
  <span class="hd-settings-label shrink-0">Opacity</span>
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
  <span class="hd-settings-value w-9 shrink-0 text-right tabular-nums">
    {pct(opacity)}
  </span>
  <button
    type="button"
    class="hd-settings-pin rounded-control p-1 transition-colors"
    class:is-on={alwaysOnTop}
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
  /* ── Tone: glass (todo widget, default) ───────────────────────────── */
  .hd-settings[data-tone='glass'] {
    background: var(--hd-surface-sunken);
    color: var(--hd-text-3);
  }
  .hd-settings[data-tone='glass'] .hd-settings-label { color: var(--hd-text-3); }
  .hd-settings[data-tone='glass'] .hd-settings-value { color: var(--hd-text-2); }
  .hd-settings[data-tone='glass'] .hd-range {
    background: var(--hd-border);
  }
  .hd-settings[data-tone='glass'] .hd-range::-webkit-slider-thumb {
    background: var(--hd-accent);
    border-color: var(--hd-surface-strong);
  }
  .hd-settings[data-tone='glass'] .hd-range::-moz-range-thumb {
    background: var(--hd-accent);
    border-color: var(--hd-surface-strong);
  }
  .hd-settings[data-tone='glass'] .hd-settings-pin { color: var(--hd-text-2); }
  .hd-settings[data-tone='glass'] .hd-settings-pin:hover { background: var(--hd-surface-2); }
  .hd-settings[data-tone='glass'] .hd-settings-pin.is-on { color: var(--hd-accent); }

  /* ── Tone: paper (sticky widget) ─────────────────────────────────── */
  .hd-settings[data-tone='paper'] {
    background: rgba(0, 0, 0, 0.06);
    color: rgba(0, 0, 0, 0.55);
  }
  .hd-settings[data-tone='paper'] .hd-settings-label { color: rgba(0, 0, 0, 0.5); }
  .hd-settings[data-tone='paper'] .hd-settings-value { color: rgba(0, 0, 0, 0.7); }
  .hd-settings[data-tone='paper'] .hd-range { background: rgba(0, 0, 0, 0.15); }
  .hd-settings[data-tone='paper'] .hd-range::-webkit-slider-thumb {
    background: rgba(0, 0, 0, 0.7);
    border-color: rgba(255, 255, 255, 0.8);
  }
  .hd-settings[data-tone='paper'] .hd-range::-moz-range-thumb {
    background: rgba(0, 0, 0, 0.7);
    border-color: rgba(255, 255, 255, 0.8);
  }
  .hd-settings[data-tone='paper'] .hd-settings-pin { color: rgba(0, 0, 0, 0.55); }
  .hd-settings[data-tone='paper'] .hd-settings-pin:hover { background: rgba(0, 0, 0, 0.08); }
  .hd-settings[data-tone='paper'] .hd-settings-pin.is-on { color: rgba(0, 0, 0, 0.85); }

  /* ── Slider chrome (shared) ──────────────────────────────────────── */
  .hd-range {
    appearance: none;
    height: 4px;
    border-radius: 9999px;
  }
  .hd-range::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 9999px;
    border-width: 2px;
    border-style: solid;
    box-shadow: var(--hd-shadow-sm);
    cursor: pointer;
  }
  .hd-range::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 9999px;
    border-width: 2px;
    border-style: solid;
    cursor: pointer;
  }
</style>
