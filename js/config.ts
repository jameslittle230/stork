import StorkError from "./storkError";
import { difference, plural } from "./util";

export interface Configuration {
  showProgress: boolean;
  printIndexInfo: boolean;
  showScores: boolean;
  showCloseButton: boolean;
  minimumQueryLength: number;
  forceOverwrite: boolean;
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
  forceOverwrite: false,
  onQueryUpdate: undefined,
  onResultSelected: undefined,
  onResultsHidden: undefined,
  onInputCleared: undefined
};

export function calculateOverriddenConfig(
  overrides: Partial<Configuration>
): Configuration | StorkError {
  const configKeyDiff = difference(
    Object.keys(overrides),
    Object.keys(defaultConfig)
  );

  if (configKeyDiff.length > 0) {
    const keys = plural(configKeyDiff.length, "key", "keys");
    const invalidKeys = JSON.stringify(configKeyDiff);
    return new StorkError(`Invalid ${keys} in config object: ${invalidKeys}`);
  }

  const output: Configuration = Object.assign({}, defaultConfig);

  for (const key of Object.keys(defaultConfig) as Array<keyof Configuration>) {
    const overrideVal = overrides[key];
    if (overrideVal !== undefined) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      output[key] = overrideVal;
    }
  }

  return output;
}
