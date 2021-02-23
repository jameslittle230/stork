import { Entity } from "./entity";
import { defaultConfig } from "./config";
import { EntityDom } from "./entityDom";
import { JSDOM } from "jsdom";
import { add } from "./dom";

import { createMockHtmlElement, MockHtmlElement } from "./__mocks__/dom";

jest.mock("./resultToListItem");
jest.mock("./wasmQueue");
jest.mock("./dom");

// @TODO: Mock resultToListItem()

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
      case `input[data-stork="test"]`:
        return mockInputElement;

      case `div[data-stork="test-output"]`:
        return mockOutputElement;
    }
  });

describe("entitydom", () => {
  let entity: Entity;
  let entityDom: EntityDom;

  beforeEach(() => {
    entity = new Entity("test", "https://google.com", defaultConfig);
    entity.attachToDom();
    entityDom = <EntityDom>entity.domManager;
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
      message: "sup",
      state: "ready"
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
      message: "sup",
      state: "ready"
    });

    // message, results list, list item, attribution, close button
    expect(add).toHaveBeenCalledTimes(0);

    expect(mockOutputElement.children.length).toEqual(0);
    expect(mockOutputElement.classList.add).not.toHaveBeenCalled();
  });

  test("calling changeHighlightedResult", () => {
    (add as jest.MockedFunction<typeof add>).mockClear();

    // Just force entity.results.length to be 2
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    entity.results = ["a", "b"];
    entityDom.render({
      resultsVisible: true,
      showProgress: true,
      showScores: true,
      progress: 0.5,
      state: "ready",
      message: "sup",
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
        },
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
      ]
    });

    // console.log(entityDom.elements.list.children.length);
    const highlightTarget = 1;
    entityDom.changeHighlightedResult({
      to: highlightTarget,
      shouldScrollTo: true
    });

    expect.assertions(entityDom.elements.list.children.length * 2);
    ((entityDom.elements.list as unknown) as MockHtmlElement).children.forEach(
      (listItem: MockHtmlElement, idx: number) => {
        const mockAddFunction = (listItem.classList
          .add as unknown) as jest.MockedFunction<typeof add>;

        const mockRemoveFunction = (listItem.classList
          .remove as unknown) as jest.MockedFunction<typeof add>;

        if (idx == highlightTarget) {
          expect(mockRemoveFunction).not.toHaveBeenCalled();
          expect(mockAddFunction).toHaveBeenCalledTimes(1);
        } else {
          expect(mockRemoveFunction).toHaveBeenCalled();
          expect(mockAddFunction).not.toHaveBeenCalled();
        }
      }
    );
  });
});
