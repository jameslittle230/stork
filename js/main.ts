import { Configuration } from "./config";
import {
  register as registerEntity,
  attachToDom,
  entityIsReady,
  debug as entityDebug
} from "./entityManager";
import { loadWasm, debug as wasmDebug } from "./wasmManager";
import { resolveSearch, SearchData } from "./searchData";
import StorkError from "./storkError";
import { validateIndexParams } from "./validators/indexParamValidator";
import { wasm_stork_version } from "stork-search";

function initialize(wasmOverrideUrl: string | null = null): Promise<void> {
  return loadWasm(wasmOverrideUrl)
    .then(() => {
      return;
    })
    .catch(() => {
      // Send error to entity manager
      throw new StorkError(
        `Can't load WASM from URL ${wasmOverrideUrl || "<no url given>"}`
      );
    });
}

function downloadIndex(name: string, url: string, config = {}): Promise<void> {
  return new Promise((res, rej) => {
    const validationError = validateIndexParams(name, url);
    if (validationError) {
      rej(validationError);
      return;
    }

    registerEntity(name, url, config).then(res).catch(rej);
  });
}

function attach(name: string): void {
  try {
    attachToDom(name);
  } catch (e) {
    throw new StorkError(e.message);
  }
}

function register(
  name: string,
  url: string,
  config: Partial<Configuration> = {}
): Promise<void> {
  const initPromise = initialize();
  const downloadPromise = downloadIndex(name, url, config);
  attach(name);

  // This silly `then` call turns a [(void), (void)] into a (void), which is
  // only necessary to make Typescript happy.
  // You begin to wonder if you write Typescript code, or if Typescript code writes you.
  return Promise.all([initPromise, downloadPromise]).then();
}

function search(name: string, query: string): SearchData {
  if (!name || !query) {
    throw new StorkError(
      "Make sure to call stork.search() with two arguments: the index name and the search query."
    );
  }

  if (!entityIsReady(name)) {
    throw new StorkError(
      "Couldn't find index. Make sure the stork.downloadIndex() promise has resolved before calling stork.search()."
    );
  }

  return resolveSearch(name, query);
}

function debug(): Record<string, unknown> {
  return {
    ...wasmDebug(),
    ...entityDebug(),
    jsStorkVersion: process.env.VERSION,
    wasmStorkVersion: wasm_stork_version()
  };
}

export { initialize, downloadIndex, attach, search, register, debug };
