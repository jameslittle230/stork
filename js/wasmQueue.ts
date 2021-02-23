export default class WasmQueue {
  queue: { (): void }[] = [];
  flushes = 0;

  /**
   * Caller should use this to queue up a function to be run only when the
   * WASM is loaded. If the WASM is already loaded when this method is called,
   * the function will run immediately.
   *
   * @param fn Function to be run once WASM is loaded
   */
  runAfterWasmLoaded(fn: { (): void; (): void }): WasmQueue {
    if (this.flushes > 0) {
      fn();
    } else {
      this.queue.push(fn);
    }

    return this;
  }

  /**
   * WASM loader should use this to signal to the queue that the WASM has been
   * loaded.
   */
  public flush(): void {
    this.flushes++;
    for (const fn of this.queue) {
      fn();
    }

    this.queue = [];
  }
}
