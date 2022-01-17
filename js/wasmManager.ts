import init from "stork-search";
import StorkError from "./storkError";

const version = process.env.VERSION;
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/releases/v${version}/stork.wasm`
  : `https://files.stork-search.net/stork.wasm`;

let wasmSourceUrl: string | null = null; // only for debug output
let wasmLoadPromise: Promise<string | void> | null = null;

let queue: { (): void }[] = [];
let errorQueue: { (): void }[] = [];

const loadWasm = (
  overrideUrl: string | null = null
): Promise<string | void> => {
  // If there's a WASM load in flight or complete, don't try to call init again
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
    .catch(() => {
      errorFlush();
      throw new StorkError(`Error while loading WASM at ${url}`);
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
 *
 * @returns a promise if loadWasm has been called, or undefined if loadWasm
 * has not been called. If loadWasm has been called, the promise will resolve
 * when the WASM has been loaded and when the function has been run.
 */
const runAfterWasmLoaded = (
  fn: () => void,
  err: () => void
): Promise<string | void> | null => {
  if (!wasmLoadPromise) {
    queue.push(fn);
    errorQueue.push(err);
    return null;
  } else {
    // We have a wasmLoadPromise, but we don't know if it's resolved.
    // Let's wait for it to resolve, then run the function.
    wasmLoadPromise.then(() => fn()).catch(() => err());
    return wasmLoadPromise;
  }
};

const flush = () => {
  queue.forEach(fn => {
    fn();
  });
  queue = [];
};

const errorFlush = () => {
  errorQueue.forEach(fn => {
    fn();
  });
  errorQueue = [];
};

const debug = (): Record<string, unknown> => ({
  wasmSourceUrl,
  wasmLoadPromise,
  queueLength: queue.length
});

export { runAfterWasmLoaded, loadWasm, debug };
