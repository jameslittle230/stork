import { Entity } from "./entity";
import { defaultConfig } from "./config";
import WasmQueue from "./wasmQueue";
import { EntityDom } from "./entityDom";
import { JSDOM } from "jsdom";

import { mockHtmlElement } from "./__mocks__/dom";

jest.mock("./wasmQueue");
jest.mock("./dom");

// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", () => {}, { virtual: true });

const dom = new JSDOM();
global.document = dom.window.document;

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
global.window = dom.window;
global.document.querySelector = jest
  .fn()
  .mockImplementation(() => mockHtmlElement);

describe("entitydom", () => {
  let entity;
  let entityDom: EntityDom;
  // Applies only to tests in this describe block
  beforeEach(() => {
    entity = new Entity(
      "test",
      "https://google.com",
      defaultConfig,
      new WasmQueue()
    );
    entityDom = entity.domManager;
  });

  test("entityDom successfully constructed", () => {
    expect(entityDom).not.toBeNull();
  });

  test("entityDom can render well", () => {
    entityDom.render({
      results: [
        {
          entry: { fields: {}, title: "title", url: "https://jameslittle.me" },
          excerpts: [],
          score: 10,
          title_highlight_ranges: []
        }
      ],
      resultsVisible: true,
      showProgress: true,
      showScores: true,
      progress: 0.5,
      message: "sup"
    });
    expect(true).not.toBeNull();
  });
});
