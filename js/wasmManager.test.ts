/* eslint-disable @typescript-eslint/no-explicit-any */
let loadWasm: any, runAfterWasmLoaded: any;

jest.mock("stork-search", () => ({
  default: jest.fn().mockImplementation(
    url =>
      new Promise(resolve => {
        resolve(url);
      })
  )
}));

describe("wasmManager", () => {
  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    jest.resetModules();
    return import("./wasmManager").then(module => {
      loadWasm = module.loadWasm;
      runAfterWasmLoaded = module.runAfterWasmLoaded;
    });
  });
  test("should load from the default URL", async () => {
    const wasm = await loadWasm();
    expect(wasm).toEqual("https://files.stork-search.net/stork.wasm");
  });

  test("Should load from a non-standard URL", async () => {
    const wasm = await loadWasm("https://example.com/stork.wasm");
    expect(wasm).toEqual("https://example.com/stork.wasm");
  });

  test("Should run a function immediately if the wasm is loaded", async () => {
    loadWasm();
    const spy = jest.fn();
    await runAfterWasmLoaded(spy);
    expect(spy).toHaveBeenCalled();
  });

  test("Should run a function only once the wasm is loaded", async () => {
    const spy = jest.fn();
    runAfterWasmLoaded(spy);
    expect(spy).not.toHaveBeenCalled();
    await loadWasm();
    expect(spy).toHaveBeenCalled();
  });
});
