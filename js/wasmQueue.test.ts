import WasmQueue from "./wasmQueue";

// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", undefined, { virtual: true });

test("WasmQueue %s %s", async () => {
  const successFxns = [jest.fn(), jest.fn()];
  const queue = new WasmQueue()
    .runAfterWasmLoaded(successFxns[0])
    .runAfterWasmLoaded(successFxns[1]);

  queue.flush();

  const computed = successFxns.map(
    // Did the function get called exactly once?
    fn => fn.mock.calls.length === 1
  );

  expect(computed).toEqual([true, true]);
});
