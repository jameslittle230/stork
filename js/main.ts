import { Configuration } from "./config";
import { EntityManager } from "./entityManager";
import WasmQueue from "./wasmQueue";
import { resolveSearch, SearchData } from "./searchData";

class StorkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StorkError";
  }
}

let wasmQueue: WasmQueue | null = null;
let entityManager: EntityManager | null = null;

function initialize(wasmOverrideUrl: string | null = null): Promise<void> {
  return new Promise((res, rej) => {
    if (!wasmQueue) {
      wasmQueue = new WasmQueue(wasmOverrideUrl)
        .runAfterWasmLoaded(res)
        .runOnWasmLoadFailure(rej);
    } else if (wasmQueue.state === "failed") {
      rej();
    } else {
      res();
    }
  });
}

function downloadIndex(name: string, url: string, config = {}): Promise<void> {
  return new Promise((res, rej) => {
    let message = null;

    if (typeof name !== "string") {
      message = "Index registration name must be a string.";
    }

    if (typeof url !== "string") {
      message = "URL must be a string.";
    }

    if (!wasmQueue) {
      message =
        "Make sure to call stork.initialize() before calling stork.downloadIndex()";
    }

    if (message) {
      const error = new StorkError(message);
      rej(error);
      return;
    }

    if (!entityManager) {
      entityManager = new EntityManager(<WasmQueue>wasmQueue);
    }

    entityManager.register(name, url, config).then(res).catch(rej);
  });
}

function attach(name: string): void {
  if (!entityManager) {
    throw new StorkError(
      "Make sure to call stork.downloadIndex() successfully before calling stork.attach()"
    );
  }

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

  // This silly then block turns a [(void), (void)] into a (void), which is
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

  if (
    !entityManager ||
    !entityManager.entities[name] ||
    entityManager.entities[name].progress < 1
  ) {
    throw new StorkError(
      "Couldn't find index. Make sure the stork.downloadIndex() promise has resolved before calling stork.search()."
    );
  }

  return resolveSearch(name, query);
}

export { initialize, downloadIndex, attach, search, register };
