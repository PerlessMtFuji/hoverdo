// Svelte 5 action: fire a handler when a click lands outside the element.
// Used by popovers (ReminderControl, future menus) so they dismiss without
// each component re-implementing the same listener boilerplate.

export function clickOutside(node: HTMLElement, handler: () => void) {
  function onClick(e: MouseEvent) {
    const target = e.target as Node | null;
    if (target && !node.contains(target)) {
      handler();
    }
  }
  // Listen on mousedown so the handler runs before any focus shift / blur,
  // matching the "click happened elsewhere first" mental model.
  document.addEventListener('mousedown', onClick, true);
  return {
    destroy() {
      document.removeEventListener('mousedown', onClick, true);
    },
    update(next: () => void) {
      handler = next;
    }
  };
}
