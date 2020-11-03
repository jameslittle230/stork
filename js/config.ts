export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
  minimumQueryLength: number;
  onQueryUpdate?: (query: string, results: unknown) => unknown;
  onResultSelected?: (query: string, result: unknown) => unknown;
}

export const defaultConfig: Readonly<Configuration> = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false,
  minimumQueryLength: 3,
  onQueryUpdate: undefined,
  onResultSelected: undefined
};

export function calculateOverriddenConfig(
  overrides: Partial<Configuration>
): Configuration {
  const output: Configuration = Object.assign({}, defaultConfig);

  for (const key of Object.keys(defaultConfig) as Array<keyof Configuration>) {
    const overrideVal = overrides[key];
    if (overrideVal !== undefined) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      //@ts-ignore
      output[key] = overrideVal;
    }
  }

  return output;
}
