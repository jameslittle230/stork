export const wasm_search = jest.fn;

export const init_spy = jest.fn().mockImplementation((input: string) => {
  return new Promise((res, rej) => {
    console.log(4, "mock stork search", input);
    if (input.includes("stork-search.net") || input.includes("example.com")) {
      res("stork-search.net");
      return;
    } else {
      rej();
    }
  });
});

export default init_spy;
