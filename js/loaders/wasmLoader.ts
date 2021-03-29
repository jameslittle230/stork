import init from "stork-search";

const version = null; // process.env.VERSION
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/stork-${version}.wasm`
  : `https://files.stork-search.net/stork.wasm`;

export const loadWasm = (overrideUrl: string | null): Promise<string> => {
  const url = overrideUrl || DEFAULT_WASM_URL;
  return init(url).then(() => {
    // Suppress the value that init succeeds with.
    return url;
  });
};
