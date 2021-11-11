import { loadWasm, runAfterWasmLoaded } from "./wasmManager";

describe("wasmManager", () => {
  test("should load from the default URL", async () => {
    const wasm = await loadWasm();
    expect(wasm).toEqual("https://files.stork-search.net/stork.wasm");
  });

  test("Should load from a non-standard URL", async () => {
    const wasm = await loadWasm("https://example.com/stork.wasm");
    expect(wasm).toEqual("https://example.com/stork.wasm");
  });

  test("Should run a function after the wasm is loaded", async () => {
    await loadWasm();
    const spy = jest.fn();
    runAfterWasmLoaded(spy);
    expect(spy).toHaveBeenCalled();
  });

  test("Should run a function after the wasm is loaded", async () => {
    const spy = jest.fn();
    runAfterWasmLoaded(spy);
    expect(spy).not.toHaveBeenCalled();
    await loadWasm();
    expect(spy).toHaveBeenCalled();
  });
});
