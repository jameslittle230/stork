// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const debugAssert = (assertion: any | boolean, ...args: unknown[]) => {
  if (false) {
    // @ts-ignore
    console.assert(assertion, ...args);
  }
};
