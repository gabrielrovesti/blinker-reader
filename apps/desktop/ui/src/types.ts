// Shared UI types for IPC and state

export interface LibraryItem {
  id: string;
  path: string;
  title: string;
  author?: string;
  file_type: string;
  hash: string;
  tags: string[];
}

export interface ScanReport {
  total: number;
  new: number;
  updated: number;
  errors: string[];
}

export interface ReaderSession {
  id: string;
  document_id: string;
  current_page: number;
  total_pages: number;
}

export interface SearchMatch {
  page: number;
  text: string;
  position: [number, number];
}

export interface Annotation {
  id: string;
  item_id: string;
  page: number;
  range: [number, number, number, number];
  kind: string;
  text: string;
  color: string;
  created_at: number;
}

