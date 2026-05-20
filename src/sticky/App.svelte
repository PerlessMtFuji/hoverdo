<script lang="ts">
  // Sticky note widget - MVP scaffold.
  // Reads ?widget_id=... from the URL when launched as a Tauri window.
  // Backend wiring (load/save via IPC) lands once the DB layer is in.

  const params = new URLSearchParams(window.location.search);
  const widgetId = params.get('widget_id') ?? 'preview';

  let body = $state('');
</script>

<div class="flex h-full flex-col">
  <header
    class="drag-region flex h-7 shrink-0 items-center justify-between px-2 text-xs text-text-3"
  >
    <span class="truncate">Sticky · {widgetId}</span>
  </header>

  <textarea
    bind:value={body}
    placeholder="Write a quick thought..."
    class="flex-1 resize-none border-0 bg-transparent p-3 text-sm leading-relaxed text-text-1 placeholder:text-text-3 focus:outline-none"
  ></textarea>
</div>
