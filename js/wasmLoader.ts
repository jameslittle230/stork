import init from "stork-search";
import WasmQueue from "./wasmQueue";

// const version = process.env.VERSION;
const version = null;
const DEFAULT_WASM_URL = version
  ? `https://files.stork-search.net/stork-${version}.wasm`
  : `https://files.stork-search.net/stork.wasm`;

export function createWasmQueue(wasmOverrideUrl: string | null): WasmQueue {
  const wasmUrl = wasmOverrideUrl || DEFAULT_WASM_URL;
  const queue = new WasmQueue();
  init(wasmUrl)
    .then(() => {
      queue.loaded = true;
      queue.handleWasmLoad();
    })
    .catch(e => {
      queue.handleWasmFailure(e);
    });
  return queue;
}
