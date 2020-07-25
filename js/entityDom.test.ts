import { Entity } from "./entity";
import { defaultConfig } from "./config";
import WasmQueue from "./wasmQueue";
import { EntityDom } from "./entityDom";
import { JSDOM } from "jsdom";
import { add } from "./dom";

import { createMockHtmlElement } from "./__mocks__/dom";

jest.mock("./wasmQueue");
jest.mock("./dom");

// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", () => {}, { virtual: true });

const mockInputElement = createMockHtmlElement();
mockInputElement.value = "input";

const mockOutputElement = createMockHtmlElement();
mockOutputElement.value = "input";

const dom = new JSDOM();
global.document = dom.window.document;

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
global.window = dom.window;
global.document.querySelector = jest
  .fn()
  .mockImplementation((query: string) => {
    switch (query) {
      case "input[data-stork=test]":
        return mockInputElement;

      case "div[data-stork=test-output]":
        return mockOutputElement;
    }
  });

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

  test("calling render with one result + one excerpt", () => {
    (add as jest.MockedFunction<typeof add>).mockClear();
    mockInputElement.value = "query";
    entityDom.render({
      results: [
        {
          entry: {
            fields: {},
            title: "result title",
            url: "https://jameslittle.me"
          },
          excerpts: [
            {
              fields: {},
              highlight_ranges: [],
              internal_annotations: [],
              score: 10,
              text: "excerpt text"
            }
          ],
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

    // message, results list, list item, attribution, close button
    expect(add).toHaveBeenCalledTimes(5);

    const outputChildrenClassLists = mockOutputElement.children.map(
      e => e.classList.entries
    );

    expect(outputChildrenClassLists.filter(a => a.length != 1).length).toBe(0);
    expect(outputChildrenClassLists.map(a => a[0])).toEqual([
      "stork-message",
      "stork-results",
      "stork-attribution"
    ]);
    expect(mockOutputElement.children[1].children.length).toBe(1);
    expect(mockOutputElement.classList.add).toHaveBeenCalledWith(
      "stork-output-visible"
    );
  });
  test("calling render with one result + one excerpt", () => {
    (add as jest.MockedFunction<typeof add>).mockClear();
    mockOutputElement.classList.add.mockClear();

    mockInputElement.value = "";
    entityDom.render({
      results: [],
      resultsVisible: true,
      showProgress: true,
      showScores: true,
      progress: 0.5,
      message: "sup"
    });

    // message, results list, list item, attribution, close button
    expect(add).toHaveBeenCalledTimes(0);

    expect(mockOutputElement.children.length).toEqual(0);
    expect(mockOutputElement.classList.add).not.toHaveBeenCalled();
  });
});
