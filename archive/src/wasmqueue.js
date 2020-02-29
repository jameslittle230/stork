export default class WasmQueue {
  constructor() {
    this.loaded = false;
    this.queue = [];
  }

  runAfterWasmLoaded(fn) {
    if (this.loaded) {
      fn();
    } else {
      this.queue.push(fn);
    }
  }

  handleWasmLoad() {
    for (let fn of this.queue) {
      fn();
    }
  }
}
