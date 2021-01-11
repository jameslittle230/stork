import init from "stork-search";
import WasmQueue from "./wasmQueue";

const prod = process.env.NODE_ENV === "production";

const wasmUrl = prod
  ? "https://files.stork-search.net/stork.wasm"
  : "http://127.0.0.1:8025/stork.wasm";

export function createWasmQueue(): WasmQueue {
  const queue = new WasmQueue();
  init(wasmUrl).then(() => {
    queue.loaded = true;
    queue.handleWasmLoad();
  });
  return queue;
}
