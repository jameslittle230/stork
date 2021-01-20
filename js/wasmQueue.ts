type WasmQueueState = "queueing" | "loaded" | "failed";

export default class WasmQueue {
  state: WasmQueueState = "queueing";
  loaded = false;
  queue: { (): void }[] = [];
  failureMethod: { (e: Error): void };

  /**
   * Caller should use this to queue up a function to be run only when the
   * WASM is loaded. If the WASM is already loaded when this method is called,
   * the function will run immediately.
   *
   * @param fn Function to be run once WASM is loaded
   */
  runAfterWasmLoaded(fn: { (): void; (): void }): void {
    if (this.loaded) {
      fn();
    } else {
      this.queue.push(fn);
    }
  }

  /**
   * Caller should use this to register a function to be run only when the WASM
   * fails to load. Unlike the happy-path function, calling this will overwrite
   * any existing error handler function.
   *
   * @param fn The function to be called when the WASM fails to load. The function
   * should take an optional Error parameter.
   */
  runOnWasmLoadFailure(fn: { (e: Error | null): void }): void {
    if (this.state === "failed") {
      fn(null);
    } else {
      this.failureMethod = fn;
    }
  }

  /**
   * WASM loader should use this to signal to the queue that the WASM has been
   * loaded.
   */
  handleWasmLoad(): void {
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
  handleWasmFailure(e: Error): void {
    if (this.failureMethod) this.failureMethod(e);
  }
}
