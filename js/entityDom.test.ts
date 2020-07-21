import { Entity } from "./entity";
import { defaultConfig } from "./config";
import WasmQueue from "./wasmQueue";
import { EntityDom } from "./entityDom";
import { JSDOM } from "jsdom";

jest.mock("./wasmQueue");
jest.mock("./dom");

// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", () => {}, { virtual: true });

const mockHtmlElement = {
  addEventListener: jest.fn(),
  insertAdjacentElement: jest.fn(),
  innerHTML: ""
};

const dom = new JSDOM();
global.document = dom.window.document;

// @ts-ignore
global.window = dom.window;
global.document.querySelector = jest
  .fn()
  .mockImplementation(queryString => mockHtmlElement);

describe("entitydom", () => {
  var entity;
  var entityDom: EntityDom;
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

  // test('San Juan <3 plantains', () => {
  //   expect(isValidCityFoodPair('San Juan', 'Mofongo')).toBe(true);
  // });
});
