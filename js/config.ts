import { difference } from "./util";

export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
  showCloseButton: boolean;
  minimumQueryLength: number;
  onQueryUpdate?: (query: string, results: unknown) => unknown;
  onResultSelected?: (query: string, result: unknown) => unknown;
  onResultsHidden?: () => unknown;
  onInputCleared?: () => unknown;
}

export const defaultConfig: Readonly<Configuration> = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false,
  showCloseButton: true,
  minimumQueryLength: 3,
  onQueryUpdate: undefined,
  onResultSelected: undefined,
  onResultsHidden: undefined,
  onInputCleared: undefined
};

export function calculateOverriddenConfig(
  overrides: Partial<Configuration>
): Configuration {
  const configKeyDiff = difference(
    Object.keys(overrides),
    Object.keys(defaultConfig)
  );

  if (configKeyDiff.length > 0) {
    const plural = (count: number, singular: string, plural: string) =>
      count == 1 ? singular : plural;
    throw new Error(
      `Invalid ${plural(
        configKeyDiff.length,
        "key",
        "keys"
      )} in config object: ${JSON.stringify(Array.from(configKeyDiff))}`
    );
  }

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
