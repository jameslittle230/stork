import * as checkeasy from "checkeasy";

import StorkError from "./storkError";

export type Configuration = ReturnType<typeof resolveConfig>;

export const resolveConfig = (object: unknown) => {
  const isFunction =
    <T>(): checkeasy.Validator<T> =>
    (v, path) => {
      if (typeof v !== "function") {
        throw new Error(`[${path}] isn't the same as allowed value`);
      }
      return v;
    };

  const validator = checkeasy.object({
    showProgress: checkeasy.defaultValue(true, checkeasy.boolean()),
    printIndexInfo: checkeasy.defaultValue(false, checkeasy.boolean()),
    showScores: checkeasy.defaultValue(false, checkeasy.boolean()),
    showCloseButton: checkeasy.defaultValue(true, checkeasy.boolean()),
    minimumQueryLength: checkeasy.defaultValue(3, checkeasy.int()),
    forceOverwrite: checkeasy.defaultValue(false, checkeasy.boolean()),
    resultNoun: checkeasy.defaultValue(
      { singular: "file", plural: "files" },
      checkeasy.object({
        singular: checkeasy.string(),
        plural: checkeasy.string()
      })
    ),
    onQueryUpdate: checkeasy.defaultValue((_query: string, _results: any) => {}, isFunction()),
    onResultSelected: checkeasy.defaultValue((_query: string, _results: any) => {}, isFunction()),
    onResultsHidden: checkeasy.defaultValue(() => {}, isFunction()),
    onInputCleared: checkeasy.defaultValue(() => {}, isFunction()),
    transformResultUrl: checkeasy.defaultValue((url: string) => url, isFunction())
  });

  try {
    return validator(object || {}, "");
  } catch (err) {
    if (err instanceof checkeasy.ValidationError) {
      throw new StorkError(err.message);
    } else {
      throw err;
    }
  }
};
