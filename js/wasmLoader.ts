import { default as wasmInit, wasm_stork_version } from "stork-search";

import StorkError from "./storkError";
import { log } from "./util/storkLog";

const version = process.env.VERSION;
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/releases/v${version}/stork.wasm`
  : `https://files.stork-search.net/stork.wasm`;

export type WasmLoadValue = {
  sourceUrl: string;
  version: string;
};

export default class WasmLoader {
  queue: { name: string; fn: () => void }[] = [];
  errorQueue: { name: string; fn: (e: Error) => void }[] = [];
  wasmLoadPromise: Promise<WasmLoadValue> | null = null;
  wasmIsLoaded = false;
  wasmSourceUrl = DEFAULT_WASM_URL;

  constructor() {
    log("WasmLoader class constructed");
  }

  load(overrideUrl?: string): Promise<WasmLoadValue> {
    if (this.wasmIsLoaded) {
      log("Wasm already loaded; returning known values");
      return Promise.resolve({
        sourceUrl: this.wasmSourceUrl,
        version: wasm_stork_version()
      });
    }

    if (this.wasmLoadPromise) {
      log("Wasm started loading; returning promise");
      return this.wasmLoadPromise;
    }

    if (overrideUrl) {
      this.wasmSourceUrl = overrideUrl;
    }

    // TODO: How to handle WASM already loaded?

    log(`Beginning wasmInit, fetching ${this.wasmSourceUrl}`);

    this.wasmLoadPromise = new Promise((res, rej) => {
      // Stick this in a zero-length setTimeout to get it to run after the event loop ticks
      setTimeout(() => {
        wasmInit(this.wasmSourceUrl)
          .then(() => {
            const wasmVersion = wasm_stork_version();
            if (wasmVersion != version) {
              const message = `WASM blob version (${wasmVersion}) must match the JS package version (${version}).`;
              log(message);
              throw new StorkError(message);
            }

            this.wasmIsLoaded = true;
            this.flushQueue();
            res({ sourceUrl: this.wasmSourceUrl, version: wasm_stork_version() });
          })
          .catch((e) => {
            log("Error loading WASM", e);
            this.flushErrorQueue(e);
            rej(new StorkError(`Error while loading WASM from ${this.wasmSourceUrl}`));
          });
      }, 0);
    });

    return this.wasmLoadPromise;
  }

  runAfterWasmLoaded(debugName: string, fn: () => void) {
    if (this.wasmIsLoaded) {
      fn();
    } else {
      this.queue.push({ name: debugName, fn });
    }
  }

  queueAfterWasmErrored(debugName: string, fn: () => void) {
    // TODO: Handle case where WASM is either successfully loaded or already errored
    this.errorQueue.push({ name: debugName, fn });
  }

  debug() {
    return {
      wasmSourceUrl: this.wasmSourceUrl,
      wasmLoadPromise: this.wasmLoadPromise,
      wasmIsLoaded: this.wasmIsLoaded,
      loadQueue: this.queue.map(({ name }) => name),
      errorQueue: this.queue.map(({ name }) => name)
    };
  }

  private flushQueue() {
    log(`Flushing WASM load function queue (${this.queue.length} functions)`);
    this.queue.forEach(({ fn }) => {
      fn();
    });
    this.queue = [];
  }

  private flushErrorQueue(e: Error) {
    log(`Flushing WASM error function queue (${this.errorQueue.length} functions)`);
    this.errorQueue.forEach(({ fn }) => {
      fn(e);
    });
    this.errorQueue = [];
  }
}
