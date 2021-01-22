import init from "stork-search";

type WasmQueueState = "queueing" | "loaded" | "failed";

// const version = process.env.VERSION;
const version = null;
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/stork-${version}.wasm`
  : `https://files.stork-search.net/stork.wasm`;

export default class WasmQueue {
  private _wasmIsLoaded = false;

  public get wasmIsLoaded(): boolean {
    return this._wasmIsLoaded;
  }

  wasmUrl: string;
  wasmLoadPromise: Promise<void>;
  state: WasmQueueState = "queueing";
  queue: { (): void }[] = [];
  failureMethod: { (e: Error): void } | null = null;

  constructor(wasmOverrideUrl: string | null = null) {
    this.wasmUrl = wasmOverrideUrl || DEFAULT_WASM_URL;
    this.wasmLoadPromise = init(this.wasmUrl)
      .then(() => {
        this.handleWasmLoad();
      })
      .catch(e => {
        this.handleWasmFailure(e);
      });
  }

  /**
   * Caller should use this to queue up a function to be run only when the
   * WASM is loaded. If the WASM is already loaded when this method is called,
   * the function will run immediately.
   *
   * @param fn Function to be run once WASM is loaded
   */
  runAfterWasmLoaded(fn: { (): void; (): void }): WasmQueue {
    if (this.wasmIsLoaded) {
      fn();
    } else {
      this.queue.push(fn);
    }

    return this;
  }

  /**
   * Caller should use this to register a function to be run only when the WASM
   * fails to load. Unlike the happy-path function, calling this will overwrite
   * any existing error handler function.
   *
   * @param fn The function to be called when the WASM fails to load. The function
   * should take an optional Error parameter.
   */
  runOnWasmLoadFailure(fn: { (e: Error | null): void }): WasmQueue {
    if (this.state === "failed") {
      fn(null);
    } else {
      this.failureMethod = fn;
    }

    return this;
  }

  /**
   * WASM loader should use this to signal to the queue that the WASM has been
   * loaded.
   */
  private handleWasmLoad(): void {
    this._wasmIsLoaded = true;
    for (const fn of this.queue) {
      fn();
    }

    this.queue = [];
  }

  /**
   * WASM loader should use this to signal to the queue that loading the WASM
   * has failed.
   *
   * @param e The error that was recieved while loading the WASM.
   */
  private handleWasmFailure(e: Error): void {
    if (this.failureMethod) this.failureMethod(e);
  }
}
