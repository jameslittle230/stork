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
  internal_annotations?: Array<Record<string, unknown>>;
  highlight_ranges?: Array<HighlightRange>;
  score: number;
  text: string;
}

export interface Result {
  entry: Entry;
  excerpts: Array<Excerpt>;
  score: number;
  title_highlight_ranges?: Array<number>;
}

export interface SearchData {
  results: Array<Result>;
  total_hit_count: number;
  url_prefix: string;
}

export function resolveSearch(name: string, query: string): SearchData {
  let searchOutput = null;
  let data = null;

  try {
    searchOutput = wasm_search(name, query);
    // If wasm_search returns an error, it will return a JSON blob. Look for
    // data.error to see if this is the case.
    data = JSON.parse(searchOutput);
  } catch (e) {
    // Data has come back improperly, even beyond an error in Rust-land.
    // analytics.log(e)
    throw Error(
      "Could not parse data from wasm_search. If you see this, please file a bug: https://jil.im/storkbug " +
        searchOutput
    );
  }

  if (!data) {
    throw Error("Data was an empty object");
  }

  if (data.error) {
    throw Error(`Could not perform search: the WASM binary failed to return search results.
    You might not be serving your search index properly.
    If you think this is an error, please file a bug: https://jil.im/storkbug
    
    The WASM binary came back with:
    ${data.error}`);
  }

  return data;
}
