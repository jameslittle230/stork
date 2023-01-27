// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const debugAssert = (assertion: any | boolean, ...args: unknown[]) => {
  // eslint-disable-next-line no-constant-condition
  if (false) {
    console.assert(assertion, ...args);
  }
};
