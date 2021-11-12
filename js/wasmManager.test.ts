/* eslint-disable @typescript-eslint/no-explicit-any */
let g_loadWasm: any = null;
let g_runAfterWasmLoaded: any = null;

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
    const { loadWasm, runAfterWasmLoaded } = require("./wasmManager");
    jest.resetModules();
    g_loadWasm = loadWasm;
    g_runAfterWasmLoaded = runAfterWasmLoaded;
  });
  test("should load from the default URL", async () => {
    const wasm = await g_loadWasm();
    expect(wasm).toEqual("https://files.stork-search.net/stork.wasm");
  });

  test("Should load from a non-standard URL", async () => {
    const wasm = await g_loadWasm("https://example.com/stork.wasm");
    expect(wasm).toEqual("https://example.com/stork.wasm");
  });

  test("Should run a function immediately if the wasm is loaded", async () => {
    g_loadWasm();
    const spy = jest.fn();
    await g_runAfterWasmLoaded(spy);
    expect(spy).toHaveBeenCalled();
  });

  test("Should run a function only once the wasm is loaded", async () => {
    const spy = jest.fn();
    g_runAfterWasmLoaded(spy);
    expect(spy).not.toHaveBeenCalled();
    await g_loadWasm();
    expect(spy).toHaveBeenCalled();
  });
});
