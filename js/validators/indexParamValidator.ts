import StorkError from "../storkError";

export const validateIndexParams = (
  name: string,
  url: string
): StorkError | null => {
  if (typeof name !== "string") {
    return new StorkError("Index registration name must be a string.");
  }

  if (typeof url !== "string") {
    return new StorkError("URL must be a string.");
  }

  return null;
};
