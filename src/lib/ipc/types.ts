// Mirror of the Rust types in `src-tauri/src/models.rs`.
// Kept hand-maintained because the surface is small and codegen is overkill
// at this stage. Update both sides together.

export interface Note {
  id: string;
  title: string;
  body: string;
  color: string | null;
  created_at: string; // RFC3339
  updated_at: string;
  deleted_at: string | null;
  hlc_ts: string;
  origin_device_id: string;
}

export interface NewNote {
  title: string;
  body: string;
  color?: string | null;
}

export interface NotePatch {
  title?: string;
  body?: string;
  /** Use `null` to clear the color, omit to keep the current value. */
  color?: string | null;
}

export interface Health {
  device_id: string;
  active_widgets: number;
}

export interface List {
  id: string;
  title: string;
  color: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  hlc_ts: string;
  origin_device_id: string;
}

export interface NewList {
  title: string;
  color?: string | null;
}

export interface Task {
  id: string;
  list_id: string;
  title: string;
  done: boolean;
  due_at: string | null;
  sort_key: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  hlc_ts: string;
  origin_device_id: string;
}

export interface CreateTaskInput {
  list_id: string;
  title: string;
  due_at?: string | null;
}
