import WasmQueue from "./wasmQueue";

// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", undefined, { virtual: true });

test.each([
  [null, true, [true, true, false, false]],
  ["https://google.com", false, [false, false, false, true]]
])(
  "Test WasmQueue Loads %s %s",
  async (initValue, expectedIsLoaded, expectedMockCalls) => {
    const successFxns = [jest.fn(), jest.fn()];
    const failureFxns = [jest.fn(), jest.fn()];
    const queue = new WasmQueue(initValue)
      .runAfterWasmLoaded(successFxns[0])
      .runAfterWasmLoaded(successFxns[1])
      .runOnWasmLoadFailure(failureFxns[0])
      .runOnWasmLoadFailure(failureFxns[1]);

    await queue.wasmLoadPromise;

    expect(queue.wasmIsLoaded).toBe(expectedIsLoaded);

    const computed = [...successFxns, ...failureFxns].map(
      // Did the function get called exactly once?
      fn => fn.mock.calls.length === 1
    );

    expect(computed).toEqual(expectedMockCalls);
  }
);
