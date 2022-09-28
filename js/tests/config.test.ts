import { resolveConfig } from "../config";

test("allows empty object", () => {
  resolveConfig({});
});

test("allows undefined", () => {
  resolveConfig(undefined);
});

test("Throws with unknown key", () => {
  expect(() => {
    resolveConfig({ foo: "bar" });
  }).toThrow();
});
