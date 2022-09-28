// import { resolveSearch, SearchData } from "./searchData";
// import StorkError from "./storkError";
// import { validateIndexParams } from "./validators/indexParamValidator";
import { wasm_stork_version } from "stork-search";

import { Configuration, resolveConfig } from "./config";
import Entity from "./entity";
import EntityLoader, { EntityLoadValue } from "./entityLoader";
import EntityStore from "./entityStore";
import { getDebugLogs, log } from "./util/storkLog";
// import {
//   register as registerEntity,
//   attachToDom,
//   entityIsReady,
//   debug as entityDebug
// } from "./entityManager";
import WasmLoader, { WasmLoadValue } from "./wasmLoader";

const wasmLoader = new WasmLoader();
const entityStore = new EntityStore();
const entityLoader = new EntityLoader(wasmLoader);

/**
 * Loads the WASM. Promise resolves with a WasmLoadValue, which says the version
 * and source URL of the loaded WASM blob, for debugging purposes.
 */
const initialize = (wasmUrl?: string): Promise<WasmLoadValue> => {
  return wasmLoader.load(wasmUrl);
};

type IndexStatistics = object;

const downloadIndex = (
  name: string,
  url: string,
  unsafeConfig: unknown
): Promise<IndexStatistics> => {
  log(`Starting downloadIndex with ${name} at ${url}`);
  const safeConfig = resolveConfig(unsafeConfig);

  const entity = new Entity(name, url, safeConfig, entityLoader, wasmLoader);
  entityStore.insert(name, entity, safeConfig);

  return entity.loadPromise;
};

const appendChunk = (_name: string, _url: string): Promise<IndexStatistics> => {
  return Promise.resolve({});
};

const attach = (name: string): boolean => {
  log(`Starting attach with ${name}`);
  entityStore.get(name).attach();
  return true;
};

const search = (name: string, query: string, options: object): string[] => {
  log(`Starting search with ${name} for query ${query}`);
  entityStore.get(name).search(query, options);
  return [];
};

const register = (name: string, url: string, unsafeConfig: unknown) => {
  log(`Starting register with ${name}`);
  const safeConfig = resolveConfig(unsafeConfig);
  const initPromise = initialize();
  const registerPromise = downloadIndex(name, url, safeConfig);
  attach(name);

  return Promise.all([initPromise, registerPromise]).then(
    ([_wasmLoadValue, registerResult]) => registerResult
  );
};

const debug = (): object => {
  return {
    wasmLoader: wasmLoader.debug(),
    entityStore: entityStore.debug(),
    jsLibraryVersion: process.env.VERSION,
    wasmLibraryVersion: wasm_stork_version(),
    logs: getDebugLogs()
  };
};

export { initialize, register, attach, downloadIndex, appendChunk, search, debug };

// function downloadIndex(name: string, url: string, config = {}): Promise<void> {
//   return new Promise((res, rej) => {
//     const validationError = validateIndexParams(name, url);
//     if (validationError) {
//       rej(validationError);
//       return;
//     }

//     registerEntity(name, url, config).then(res).catch(rej);
//   });
// }

// function attach(name: string): void {
//   try {
//     attachToDom(name);
//   } catch (e) {
//     throw new StorkError(e.message);
//   }
// }

// function register(
//   name: string,
//   url: string,
//   config: Partial<Configuration> = {}
// ): Promise<void> {
//   const initPromise = initialize();
//   const downloadPromise = downloadIndex(name, url, config);
//   attach(name);

//   // This silly `then` call turns a [(void), (void)] into a (void), which is
//   // only necessary to make Typescript happy.
//   // You begin to wonder if you write Typescript code, or if Typescript code writes you.
//   return Promise.all([initPromise, downloadPromise]).then();
// }

// function search(name: string, query: string): SearchData {
//   if (!name || !query) {
//     throw new StorkError(
//       "Make sure to call stork.search() with two arguments: the index name and the search query."
//     );
//   }

//   if (!entityIsReady(name)) {
//     throw new StorkError(
//       "Couldn't find index. Make sure the stork.downloadIndex() promise has resolved before calling stork.search()."
//     );
//   }

//   return resolveSearch(name, query);
// }
