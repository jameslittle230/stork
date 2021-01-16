import { Configuration } from "./config";
import { createWasmQueue } from "./wasmLoader";
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

function initialize(): Promise<void> {
  return new Promise((res, _rej) => {
    if (!wasmQueue) {
      wasmQueue = createWasmQueue();
      wasmQueue.runAfterWasmLoaded(() => {
        res();
      });
    } else {
      res();
    }
  });
}

function downloadIndex(name: string, url: string, config = {}): Promise<void> {
  return new Promise((res, rej) => {
    var message = null;

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

function attach(name: string) {
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

function register(name: string, url: string, config: Partial<Configuration>) {
  initialize();
  downloadIndex(name, url, config);
  attach(name);
}

function search(name: string, query: string): SearchData {
  return resolveSearch(name, query);
}

export { initialize, downloadIndex, attach, search, register };
