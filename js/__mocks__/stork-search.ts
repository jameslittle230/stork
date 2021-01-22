export const wasm_search = jest.fn;
export default jest.fn().mockImplementation((input: string) => {
  return new Promise((res, rej) => {
    if (input.includes("stork-search.net")) {
      res();
      return;
    } else {
      rej();
    }
  });
});
