export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
}

const defaultConfig: Readonly<Configuration> = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false
};

function assertValidConfigurationKey(
  key: string
): asserts key is keyof Configuration {
  if (!(key in defaultConfig)) {
    throw new Error();
  }
}

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
