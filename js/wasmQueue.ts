export default class WasmQueue {
  loaded = false;
  queue: { (): void }[] = [];

  runAfterWasmLoaded(fn: { (): void; (): void }): void {
    if (this.loaded) {
      fn();
    } else {
      this.queue.push(fn);
    }
  }

  handleWasmLoad(): void {
    for (const fn of this.queue) {
      fn();
    }

    this.queue = [];
  }
}
