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
  original: Configuration,
  overrides: Partial<Configuration>
): Configuration {
  const output: Configuration = defaultConfig;

  Object.keys(overrides).forEach((key: string) => {
    try {
      assertValidConfigurationKey(key);
    } catch (error) {
      return;
    }

    output[key] = overrides[key] || output[key];
  });

  return output;
}
