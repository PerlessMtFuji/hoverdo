// Widgets store - tracks the set of active widget instances so the home
// hub gallery can update reactively. Tiny on purpose: list_widgets is
// cheap and we only mutate via pin/unpin RPCs.

import * as widgetsApi from '$lib/ipc/widgets';
import { on } from '$lib/ipc/events';

class WidgetsStore {
  widgets = $state<widgetsApi.WidgetInstance[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  private subscribed = false;

  async ensureSubscribed(): Promise<void> {
    if (this.subscribed) return;
    this.subscribed = true;
    try {
      await on('widgets:changed', () => {
        void this.refresh();
      });
    } catch (e) {
      this.subscribed = false;
      console.warn('failed to subscribe to widgets:changed', e);
    }
  }

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.widgets = await widgetsApi.listWidgets();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }
}

export const widgetsStore = new WidgetsStore();
