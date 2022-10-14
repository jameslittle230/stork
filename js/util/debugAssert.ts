// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const debugAssert = (assertion: any | boolean, ...args: unknown[]) => {
  console.assert(assertion, ...args);
};
