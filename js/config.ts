export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
}

export const defaultConfig: Readonly<Configuration> = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false
};

export function calculateOverriddenConfig(
  overrides: Partial<Configuration>
): Configuration {
  const output: Configuration = defaultConfig;

  for (const key of Object.keys(defaultConfig) as Array<keyof Configuration>) {
    if (overrides[key]) {
      const overrideVal = overrides[key] as boolean;
      output[key] = overrideVal;
    }
  }

  return output;
}
