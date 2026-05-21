// Typed listeners for app-wide Tauri events. Wrapping the raw `listen` call
// here keeps the event names in one place and gives callers a typed payload.

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type EventMap = {
  'notes:changed': string; // id of the affected note
  'lists:changed': string; // id of the affected list
  'tasks:changed': string; // list_id of the affected task's list
  'reminders:changed': string; // list_id of the affected task's list
  'widgets:changed': null; // active widget set changed; payload unused
};

export function on<E extends keyof EventMap>(
  name: E,
  handler: (payload: EventMap[E]) => void
): Promise<UnlistenFn> {
  return listen<EventMap[E]>(name, (e) => handler(e.payload));
}
