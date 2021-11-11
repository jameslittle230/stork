import { attachToDom, register } from "./entityManager";

jest.mock("./loaders/indexLoader", () => {
  return {
    loadIndexFromUrl: jest.fn().mockImplementation((_url, { load }) => {
      load();
    })
  };
});

jest.mock("./wasmManager", () => ({
  runAfterWasmLoaded: jest.fn().mockImplementation(fn => {
    fn();
  })
}));

jest.mock("./entity");

describe("entityManager", () => {
  test("can't insert two indexes with the same name", () => {
    expect.assertions(1);
    register("index-name", "", {}).then(() =>
      register("index-name", "", {}).catch(e => expect(e).toBeTruthy())
    );
  });

  test("attachToDom fails with missing index", () => {
    expect(() => attachToDom("doesnt-exist")).toThrow();
  });
});
