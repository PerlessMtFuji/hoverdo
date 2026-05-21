import { invoke } from '@tauri-apps/api/core';

export interface SearchHit {
  kind: 'note' | 'list' | 'task';
  id: string;
  title: string;
  snippet: string;
  list_id: string | null;
  rank: number;
}

export function search(query: string): Promise<SearchHit[]> {
  return invoke('search', { query });
}
