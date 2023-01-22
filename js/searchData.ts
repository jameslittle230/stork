export interface HighlightRange {
  beginning: number;
  end: number;
}

export interface Entry {
  fields: Record<string, unknown>;
  title: string;
  url: string;
}

export interface Excerpt {
  fields: Record<string, unknown>;
  internal_annotations?: Array<Record<string, unknown>>;
  highlight_ranges?: Array<HighlightRange>;
  score: number;
  text: string;
}

export interface Result {
  entry: Entry;
  excerpts: Array<Excerpt>;
  score: number;
  title_highlight_ranges?: Array<HighlightRange>;
}

export interface SearchValue {
  results: Array<Result>;
  total_hit_count: number;
  url_prefix: string;
}
