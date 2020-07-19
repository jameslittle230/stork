import { wasm_search } from "stork-search";

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
  internal_annotations: Array<Record<string, unknown>>;
  highlight_ranges: Array<HighlightRange>;
  score: number;
  text: string;
}

export interface Result {
  entry: Entry;
  excerpts: Array<Excerpt>;
  score: number;
  title_highlight_ranges: Array<number>;
}

export interface SearchData {
  results: Array<Result>;
  total_hit_count: number;
  url_prefix: string;
}

export async function resolveSearch(
  index: Uint8Array,
  query: string
): Promise<SearchData> {
  try {
    const data = JSON.parse(wasm_search(index, query));

    if (!data) {
      throw Error("Data was an empty object");
    }
    return data;
  } catch (e) {
    // Data has come back improperly
    // analytics.log(e)
    throw Error("Could not parse data from wasm_search");
  }
}
