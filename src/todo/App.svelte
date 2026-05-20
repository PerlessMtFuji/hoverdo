<script lang="ts">
  // To-do list widget - MVP scaffold.
  // Backend wiring (load/save via IPC, drag-reorder, due dates) lands once the
  // DB + IPC layers are in.

  type LocalTask = { id: string; title: string; done: boolean };

  const params = new URLSearchParams(window.location.search);
  const widgetId = params.get('widget_id') ?? 'preview';

  let tasks = $state<LocalTask[]>([]);
  let draft = $state('');

  function add() {
    const title = draft.trim();
    if (!title) return;
    tasks.push({ id: crypto.randomUUID(), title, done: false });
    draft = '';
  }
</script>

<div class="flex h-full flex-col">
  <header
    class="drag-region flex h-7 shrink-0 items-center justify-between px-2 text-xs text-text-3"
  >
    <span class="truncate">To-do · {widgetId}</span>
  </header>

  <ul class="flex-1 overflow-auto px-2 pb-2">
    {#each tasks as task (task.id)}
      <li
        class="flex items-center gap-2 rounded-control px-2 py-1.5 hover:bg-surface-2"
      >
        <input
          type="checkbox"
          bind:checked={task.done}
          class="size-4 accent-accent"
        />
        <span
          class="flex-1 text-sm"
          class:line-through={task.done}
          class:text-text-3={task.done}
        >
          {task.title}
        </span>
      </li>
    {/each}
  </ul>

  <form
    class="flex shrink-0 items-center gap-2 border-t border-border-subtle p-2"
    onsubmit={(e) => {
      e.preventDefault();
      add();
    }}
  >
    <input
      bind:value={draft}
      type="text"
      placeholder="Add task..."
      class="flex-1 rounded-control bg-surface-1 px-2 py-1 text-sm text-text-1 placeholder:text-text-3 focus:outline-none"
    />
  </form>
</div>
