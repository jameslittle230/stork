export const defaultConfig = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false
};

export function calculateOverriddenConfig(original, overrides) {
  const output = Object.create({});
  Object.keys(original).forEach(key => {
    if (key in overrides) {
      output[key] = overrides[key];
    } else {
      output[key] = original[key];
    }
  });
  return output;
}
