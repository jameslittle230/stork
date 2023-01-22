import { wasm_stork_version } from "stork-search";

import { resolveRegisterConfig, resolveUIConfig } from "./config";
import Entity from "./entity";
import EntityStore from "./entityStore";
import IndexLoader from "./indexLoader";
import StorkError from "./storkError";
import { getDebugLogs, log } from "./util/storkLog";
import WasmLoader, { WasmLoadValue } from "./wasmLoader";

let wasmLoader: WasmLoader | null = null;
let indexLoader: IndexLoader | null = null;

const entityStore = new EntityStore();

/**
 * Loads the WASM blob.
 *
 * The returned promise resolves with a WasmLoadValue, which contains
 * the version and source URL of the loaded WASM blob, for debugging purposes.
 *
 * If the promise fails, the WASM blob was not loaded successfully, and you
 * are able to call `stork.initialize()` again.
 *
 * If the promise succeeds, any subsequent calls to `stork.initialize()` will
 * not have any effect. The returned promise will resolve immediately with the
 * loaded WASM blob's debugging information.
 *
 * If `stork.initialize()` is called before the previous invocation's promise
 * completes, the function will return with the in-flight promise.
 */
const initialize = (wasmUrl?: string): Promise<WasmLoadValue> => {
  if (!wasmLoader) {
    wasmLoader = new WasmLoader();
  }

  return wasmLoader.load(wasmUrl);
};

type IndexStatistics = object;

/**
 * Downloads the search index file from the specified URL, and registers the
 * index with the WASM binary.
 *
 * This function returns a promise which resolves with the search index's statistics,
 * for debugging purposes. After the promise resolves, you will be able to run
 * search queries with the given index identifier, either via the `stork.search()`
 * function or via the attached <input> tag.
 *
 * This promise may fail for one of several reasons:
 *
 * - This function was called before `stork.initialize()` was called
 * - The index file could not be fetched at the given URL.
 * - The index file was downloaded, but could not be parsed by the WASM binary
 * - The WASM binary failed to load successfully, and the promise returned by
 *   `stork.initialize()` threw an error
 *
 * If the promise fails, you can try calling `stork.downloadIndex()` again.
 * If you call `stork.downloadIndex()` with the same index identifier and the
 * index file has already been downloaded, Stork will use the same index binary
 * unless `{forceRefreshIndex: true}` is included in the passed-in config.
 *
 * After the promise succeeds, any subsequent calls to `stork.downloadIndex()`
 * with the same index identifier will have no effect, unless
 * `{forceRefreshIndex: true}` is passed into the config, even if the given URL
 * is different. When `forceRefreshIndex` is true, the index will be
 * re-downloaded and reloaded into the WASM.
 *
 * If you call `stork.downloadIndex()` before a previous call with the same
 * index identifier has finished (i.e. before the promise has resolved or thrown),
 * the function will not perform any actions, and will return with the previous
 * in-flight promise.
 *
 * @param name The identifier of the index
 *
 * @param url The URL from which Stork will make an XMLHttpRequest to download the
 * index file
 *
 * @param unsafeConfig A configuration object
 */
const downloadIndex = (name: string, url: string, unsafeConfig: any): Promise<IndexStatistics> => {
  log(`Starting downloadIndex with ${name} at ${url}`);
  const safeConfig = resolveRegisterConfig(unsafeConfig);

  if (!wasmLoader) {
    throw new StorkError("Called downloadIndex before calling initialize");
  }

  if (!indexLoader) {
    indexLoader = new IndexLoader(wasmLoader);
  }

  const entity = new Entity(name, url, safeConfig, indexLoader, wasmLoader);
  entityStore.insert(name, entity, safeConfig);

  return entity.load();
};

/**
 * Documentation not available yet
 */
const appendChunk = (_name: string, _url: string): Promise<IndexStatistics> => {
  return Promise.resolve({});
};

/**
 * Harnesses existing DOM elements on the page to display live search results
 * based on the query typed into an <input> tag.
 *
 * This function will look for two elements in the current DOM. Given the index
 * identifier `myIndex`, this function will expect:
 *
 * - An <input> tag with the attribute `data-stork="myIndex"`
 * - A <div> tag with the attribute `data-stork="myIndex-output"`
 *
 * These elements can be anywhere in the current DOM tree; they don't need to be
 * near each other.
 *
 * This function doesn't return a value, but will throw if there is a problem.
 * This function will throw if:
 *
 * - It is called before `stork.downloadIndex()` is called
 * - The required elements cannot be found
 *
 * Calling this function will have no effect if a previous call to
 * `stork.attach()` with the same index identifier was successful and the
 * given elements have already been harnessed. To refresh the Stork-managed DOM,
 * either refresh the page, or:
 *
 * 1. Remove all the elements with the `data-stork` attribute, and their children
 * 2. Add back your <input> and <div> elements
 * 3. Call `stork.attach()` again
 *
 * @param name The index identifier
 */
const attach = (name: string, unsafeUIConfig: any): void => {
  log(`Starting attach with ${name}`);
  const safeUIConfig = resolveUIConfig(unsafeUIConfig);
  entityStore.get(name).attach(safeUIConfig);
};

/**
 *
 */
const search = (name: string, query: string, options: object): string[] => {
  log(`Starting search with ${name} for query ${query}`);
  entityStore.get(name).performSearch(query);
  return [];
};

const register = (name: string, url: string, config: unknown) => {
  log(`Starting register with ${name}`);
  const safeRegisterConfig = resolveRegisterConfig(config); // TODO: Check whether I can make two configs from one object or not
  const safeUIConfig = resolveUIConfig(config);
  const initPromise = initialize();
  const registerPromise = downloadIndex(name, url, safeRegisterConfig);
  attach(name, safeUIConfig);

  return Promise.all([initPromise, registerPromise]).then(
    ([_wasmLoadValue, registerResult]) => registerResult
  );
};

const debug = (): object => {
  let wasmLibraryVersion: string | null = null;

  try {
    wasmLibraryVersion = wasm_stork_version();
    // eslint-disable-next-line no-empty
  } catch (e) {}

  return {
    logs: getDebugLogs(),
    wasmLoader: wasmLoader?.debug(),
    entityStore: entityStore.debug(),
    // @ts-expect-error
    jsLibraryVersion: __VERSION,
    wasmLibraryVersion
  };
};

export { initialize, register, attach, downloadIndex, appendChunk, search, debug };
