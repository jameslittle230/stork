import { defaultConfig, calculateOverriddenConfig } from "./config";

test("correctly overrides default config", () => {
  const expected = {
    showProgress: true,
    printIndexInfo: false,
    showScores: true
  };

  const generated = calculateOverriddenConfig({
    showScores: true
  });

  expect(generated).toMatchObject(expected);
});
