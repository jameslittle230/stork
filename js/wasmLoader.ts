import { default as wasmInit, wasm_stork_version } from "stork-search";

import { LoadState } from "./loadManager";
import StorkError from "./storkError";
import { debugAssert } from "./util/debugAssert";
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
  wasmLoadPromise: Promise<WasmLoadValue> | null = null;

  // Once set to `success`, this should never be set to any other value
  loadState: LoadState = "notStarted";

  // When WASM is loaded successfully, these functions are called, then removed
  // from this array.
  queue: { name: string; fn: () => void }[] = [];

  // When WASM fails to load, these functions are called, then removed from
  // this array
  errorQueue: { name: string; fn: (e: Error) => void }[] = [];

  wasmSourceUrl = DEFAULT_WASM_URL;

  constructor() {
    log("WasmLoader class constructed");
  }

  load(overrideUrl?: string): Promise<WasmLoadValue> {
    if (this.loadState === "success") {
      log("Wasm is already loaded; returning known values");
      return Promise.resolve({
        sourceUrl: this.wasmSourceUrl,
        version: wasm_stork_version()
      });
    }

    if (this.loadState === "incomplete") {
      log("Called wasmLoader.load while existing load is in progress; returning in-flight promise");
      debugAssert(this.wasmLoadPromise);
      if (!this.wasmLoadPromise) {
        return Promise.reject("bad state");
      }
      return this.wasmLoadPromise;
    }

    if (overrideUrl) {
      this.wasmSourceUrl = overrideUrl;
    }

    log(`Beginning wasmInit, fetching ${this.wasmSourceUrl}`);

    return new Promise((res) => {
      // Stick this in a zero-length setTimeout to get it to run
      // after the event loop ticks
      setTimeout(() => {
        // TODO: Call fetch here and pass the binary directly so that we can provide
        // better diagnostics if the fetch 404s
        wasmInit(this.wasmSourceUrl)
          .then(() => {
            const wasmVersion = wasm_stork_version();

            if (wasmVersion != version) {
              const message = `WASM blob version (${wasmVersion}) must match the JS package version (${version}).`;
              log(message);
              throw new StorkError(message);
            }

            this.loadState = "success";
            this.flushQueue();

            res({ sourceUrl: this.wasmSourceUrl, version: wasmVersion });
          })
          .catch((e) => {
            log("Error loading WASM.", e);

            this.loadState = "failure";
            this.flushErrorQueue(e);

            throw new StorkError(`Error while loading WASM from ${this.wasmSourceUrl}`);
          });
      }, 0);
    });
  }

  runAfterWasmLoaded(debugName: string, fn: () => void) {
    if (this.loadState === "success") {
      fn();
    } else {
      this.queue.push({ name: debugName, fn });
    }
  }

  runAfterWasmError(debugName: string, fn: () => void) {
    // TODO: Handle case where WASM is either successfully loaded or already errored
    this.errorQueue.push({ name: debugName, fn });
  }

  debug() {
    return {
      wasmSourceUrl: this.wasmSourceUrl,
      wasmLoadPromise: this.wasmLoadPromise,
      wasmLoadState: this.loadState,
      loadQueue: this.queue.map(({ name }) => name),
      errorQueue: this.queue.map(({ name }) => name)
    };
  }

  private flushQueue() {
    log(`Flushing WASM load function queue (${this.queue.length} functions)`);
    this.queue.forEach(({ fn }) => {
      debugAssert(this.loadState === "success");
      try {
        fn();
      } catch (_e) {
        // no-op
      }
    });
    this.queue = [];
  }

  private flushErrorQueue(e: Error) {
    log(`Flushing WASM error function queue (${this.errorQueue.length} functions)`);
    this.errorQueue.forEach(({ fn }) => {
      debugAssert(this.loadState === "failure");
      fn(e);
    });
    this.errorQueue = [];
  }
}
