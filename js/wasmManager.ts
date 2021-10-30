import init from "stork-search";

const version = null; // process.env.VERSION
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/stork-${version}.wasm`
  : `https://files.stork-search.net/stork.wasm`;

let wasmSourceUrl: string | null = null; // only for debug
let wasmLoadPromise: Promise<string | void> | null = null;

let queue: { (): void }[] = [];

const loadWasm = (overrideUrl: string | null): Promise<string | void> => {
  if (wasmLoadPromise) {
    return wasmLoadPromise;
  }

  const url = overrideUrl || DEFAULT_WASM_URL;
  wasmSourceUrl = url;

  const p = init(url)
    .then(() => {
      flush();
      return url;
    })
    .catch(e => {
      console.error(e);
    });

  wasmLoadPromise = p;
  return p;
};

/**
 * Caller should use this to queue up a function to be run only when the
 * WASM is loaded. If the WASM is already loaded when this method is called,
 * the function will run immediately.
 *
 * @param fn Function to be run once WASM is loaded
 */
const runAfterWasmLoaded = (fn: () => void): void => {
  if (!wasmLoadPromise) {
    queue.push(fn);
  } else {
    wasmLoadPromise.then(() => fn());
  }
};
/**
 * WASM loader should use this to signal to the queue that the WASM has been
 * loaded.
 */
const flush = () => {
  queue.forEach(fn => {
    fn();
  });
  queue = [];
};

const debug = (): Record<string, unknown> => ({
  wasmSourceUrl,
  wasmLoadPromise,
  queueLength: queue.length
});

export { runAfterWasmLoaded, loadWasm, debug };
