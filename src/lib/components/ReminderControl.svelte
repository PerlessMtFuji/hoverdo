<script lang="ts">
  // Per-task reminder affordance: a clock button that toggles a popover
  // with quick presets + a native datetime-local input. Showing the
  // existing reminder is a chip; clicking it cancels.

  import { Bell, BellOff, BellRing } from '@lucide/svelte';
  import { listsStore } from '$lib/stores/lists.svelte';
  import type { Reminder } from '$lib/ipc/types';

  type Props = { taskId: string; reminder: Reminder | undefined };
  let { taskId, reminder }: Props = $props();

  let open = $state(false);
  // Pre-fill the picker with "in 1 hour" so quick custom edits are fast.
  let customValue = $state(defaultCustom());

  function defaultCustom(): string {
    const d = new Date(Date.now() + 60 * 60 * 1000);
    // datetime-local needs YYYY-MM-DDTHH:mm in local time, no seconds, no TZ.
    const pad = (n: number) => n.toString().padStart(2, '0');
    return (
      `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T` +
      `${pad(d.getHours())}:${pad(d.getMinutes())}`
    );
  }

  function relative(iso: string): string {
    const t = new Date(iso).getTime();
    const diff = (t - Date.now()) / 1000;
    if (diff < 0) return 'now';
    if (diff < 60) return `${Math.round(diff)}s`;
    if (diff < 3600) return `${Math.round(diff / 60)}m`;
    if (diff < 86_400) return `${Math.round(diff / 3600)}h`;
    return `${Math.round(diff / 86_400)}d`;
  }

  async function preset(deltaMs: number) {
    await listsStore.setReminderForTask(taskId, new Date(Date.now() + deltaMs));
    open = false;
  }

  async function tomorrowMorning() {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    d.setHours(9, 0, 0, 0);
    await listsStore.setReminderForTask(taskId, d);
    open = false;
  }

  async function submitCustom() {
    if (!customValue) return;
    const dt = new Date(customValue);
    if (Number.isNaN(dt.getTime())) return;
    await listsStore.setReminderForTask(taskId, dt);
    open = false;
  }

  async function cancel() {
    await listsStore.clearReminderForTask(taskId);
    open = false;
  }
</script>

<div class="relative inline-flex items-center gap-1">
  {#if reminder}
    <button
      type="button"
      class="inline-flex items-center gap-1 rounded-control bg-accent/10 px-1.5 py-0.5 text-[11px] text-accent transition-colors hover:bg-accent/15"
      title={`Reminder: ${new Date(reminder.due_at).toLocaleString()}`}
      onclick={() => (open = !open)}
    >
      <BellRing size={11} />
      {relative(reminder.due_at)}
    </button>
  {:else}
    <button
      type="button"
      class="rounded-control p-1 text-text-3 opacity-0 transition-opacity hover:bg-surface-1 hover:text-accent group-hover:opacity-100"
      aria-label="Set reminder"
      title="Set reminder"
      onclick={() => (open = !open)}
    >
      <Bell size={12} />
    </button>
  {/if}

  {#if open}
    <div
      class="absolute right-0 top-full z-20 mt-1 w-56 rounded-control border border-border-subtle bg-surface-1/95 p-2 shadow-hd-lg backdrop-blur"
      role="dialog"
    >
      <p class="px-1 pb-1 text-[10px] uppercase tracking-wider text-text-3">
        Remind me
      </p>
      <div class="grid grid-cols-3 gap-1">
        <button
          type="button"
          class="rounded-control bg-surface-2 px-2 py-1 text-[11px] text-text-1 hover:bg-accent hover:text-accent-fg"
          onclick={() => preset(15 * 60 * 1000)}
        >
          15m
        </button>
        <button
          type="button"
          class="rounded-control bg-surface-2 px-2 py-1 text-[11px] text-text-1 hover:bg-accent hover:text-accent-fg"
          onclick={() => preset(60 * 60 * 1000)}
        >
          1h
        </button>
        <button
          type="button"
          class="rounded-control bg-surface-2 px-2 py-1 text-[11px] text-text-1 hover:bg-accent hover:text-accent-fg"
          onclick={() => preset(4 * 60 * 60 * 1000)}
        >
          4h
        </button>
        <button
          type="button"
          class="col-span-3 rounded-control bg-surface-2 px-2 py-1 text-[11px] text-text-1 hover:bg-accent hover:text-accent-fg"
          onclick={tomorrowMorning}
        >
          Tomorrow 9:00
        </button>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <input
          type="datetime-local"
          bind:value={customValue}
          class="flex-1 rounded-control border border-border-subtle bg-surface-2 px-1.5 py-0.5 text-[11px] text-text-1 focus:outline-none focus:ring-1 focus:ring-accent"
        />
        <button
          type="button"
          class="rounded-control bg-accent px-2 py-0.5 text-[11px] font-medium text-accent-fg hover:brightness-105"
          onclick={submitCustom}
        >
          Set
        </button>
      </div>
      {#if reminder}
        <button
          type="button"
          class="mt-2 inline-flex w-full items-center justify-center gap-1 rounded-control px-2 py-1 text-[11px] text-danger hover:bg-danger/10"
          onclick={cancel}
        >
          <BellOff size={11} />
          Clear reminder
        </button>
      {/if}
    </div>
  {/if}
</div>
