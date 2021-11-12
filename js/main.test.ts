import { register, initialize } from "./main";
jest.mock("stork-search");

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let init: any;

import("stork-search").then(module => {
  init = module.default;
});

jest.mock("./entityManager", () => ({
  register: jest.fn().mockResolvedValue(undefined),
  attachToDom: jest.fn(),
  entityIsReady: jest.fn().mockReturnValue(true)
}));
jest.mock("./entity");

describe("main tests", () => {
  beforeEach(() => {
    jest.resetModules();
    // const m_init = init as jest.Mock;
    init.mockClear();
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    // return import("./main").then(module => {
    //   initialize = module.initialize;
    //   register = module.register;
    // });
  });

  it("should only initialize WASM once", async () => {
    // m_init.mockClear();

    // Initialize with example URL
    initialize("https://example.com/stork.wasm");

    // Call register, which if called alone would call init with
    // the default URL, but should not call init again because
    // init has already been called
    register("something", "./something.st");
    expect(init).toHaveBeenCalledTimes(1);
    expect(init).toHaveBeenLastCalledWith("https://example.com/stork.wasm");
  });

  // it("should initialize WASM once with default URL when register is called", async () => {
  //   const m_init = init as jest.Mock;
  //   // m_init.mockClear();

  //   await register("something", "./something.st");
  //   expect(init).toHaveBeenCalledTimes(1);
  //   expect(m_init.mock.calls[0][0]).toMatch(/stork-search\.net\/.*\.wasm/);
  // });
});
