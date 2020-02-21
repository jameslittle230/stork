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
    this.queue.forEach(fn => {
      fn();
    });
  }
}
