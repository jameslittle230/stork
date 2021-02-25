import { Configuration } from "./config";
import { EntityManager } from "./entityManager";
import WasmQueue from "./wasmQueue";
import { loadWasm } from "./loaders/wasmLoader";
import { resolveSearch, SearchData } from "./searchData";
import StorkError from "./storkError";
import { validateIndexParams } from "./validators/indexParamValidator";

const wasmQueue: WasmQueue = new WasmQueue();
const entityManager: EntityManager = new EntityManager(wasmQueue);

function initialize(wasmOverrideUrl: string | null = null): Promise<void> {
  return loadWasm(wasmOverrideUrl)
    .then(() => {
      wasmQueue.flush();
    })
    .catch(e => {
      // Send error to entity manager
      throw new StorkError(e);
    });
}

function downloadIndex(name: string, url: string, config = {}): Promise<void> {
  return new Promise((res, rej) => {
    const validationError = validateIndexParams(name, url);
    if (validationError) {
      rej(validationError);
      return;
    }

    entityManager.register(name, url, config).then(res).catch(rej);
  });
}

function attach(name: string): void {
  try {
    entityManager.attachToDom(name);
  } catch (e) {
    throw new StorkError(e.message);
  }
}

function register(
  name: string,
  url: string,
  config: Partial<Configuration>
): Promise<void> {
  const initPromise = initialize();
  const donwloadPromise = downloadIndex(name, url, config);
  attach(name);

  // This silly `then` call turns a [(void), (void)] into a (void), which is
  // only necessary to make Typescript happy.
  // You begin to wonder if you write Typescript code, or if Typescript code writes you.
  return Promise.all([initPromise, donwloadPromise]).then();
}

function search(name: string, query: string): SearchData {
  if (!name || !query) {
    throw new StorkError(
      "Make sure to call stork.search() with two arguments: the index name and the search query."
    );
  }

  if (entityManager.entities[name]?.state != "ready") {
    throw new StorkError(
      "Couldn't find index. Make sure the stork.downloadIndex() promise has resolved before calling stork.search()."
    );
  }

  return resolveSearch(name, query);
}

export { initialize, downloadIndex, attach, search, register };
