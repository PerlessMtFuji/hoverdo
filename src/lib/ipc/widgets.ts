// Widget IPC: pin notes as floating windows, restore, persist geometry.

import { invoke } from '@tauri-apps/api/core';

export type WidgetKind = 'sticky' | 'todo';

export interface WidgetInstance {
  id: string;
  kind: string; // 'sticky' | 'todo' (stored as string in SQLite)
  target_id: string;
  win_x: number | null;
  win_y: number | null;
  win_w: number;
  win_h: number;
  opacity: number;
  always_on_top: boolean;
  theme_override: string | null;
  last_opened_at: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  hlc_ts: string;
  origin_device_id: string;
}

export function pinNote(noteId: string): Promise<WidgetInstance> {
  return invoke('pin_note', { noteId });
}

export function unpinWidget(widgetId: string): Promise<void> {
  return invoke('unpin_widget', { widgetId });
}

export function getWidget(widgetId: string): Promise<WidgetInstance | null> {
  return invoke('get_widget', { widgetId });
}

export function listWidgets(): Promise<WidgetInstance[]> {
  return invoke('list_widgets');
}

export function saveWidgetGeometry(
  widgetId: string,
  x: number | null,
  y: number | null,
  w: number,
  h: number
): Promise<void> {
  return invoke('save_widget_geometry', { widgetId, x, y, w, h });
}
