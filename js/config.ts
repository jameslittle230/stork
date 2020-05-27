export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
  onQueryUpdate?: (query: string, results: unknown) => void;
}

export const defaultConfig: Readonly<Configuration> = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false
};

export function calculateOverriddenConfig(
  overrides: Partial<Configuration>
): Configuration {
  const output: Configuration = Object.assign({}, defaultConfig);

  for (const key of Object.keys(defaultConfig) as Array<keyof Configuration>) {
    if (overrides[key] !== undefined) {
      const overrideVal = overrides[key] as boolean;
      output[key] = overrideVal;
    }
  }

  return output;
}
