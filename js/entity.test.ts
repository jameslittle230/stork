import { Entity } from "./entity";
import { defaultConfig } from "./config";
import WasmQueue from "./wasmQueue";
jest.mock("./wasmQueue");
jest.mock("./entityDom");
// eslint-disable-next-line @typescript-eslint/no-empty-function
jest.mock("stork-search", () => {}, { virtual: true });

test("Can successfully generate an entity", () => {
  const entity = new Entity(
    "test",
    "https://google.com",
    defaultConfig,
    new WasmQueue()
  );
  expect(entity).toBeTruthy();
  entity.attachToDom();
});

test("Injest search data maps url values and calls render", () => {
  const entity = new Entity(
    "test",
    "https://google.com",
    defaultConfig,
    new WasmQueue()
  );
  entity.attachToDom();
  entity.injestSearchData({
    results: [
      {
        entry: { fields: {}, title: "bleh", url: "https://google.com" },
        excerpts: [
          {
            fields: {},
            internal_annotations: [{ a: "#suffix" }],
            highlight_ranges: [],
            score: 0,
            text: "blah"
          }
        ],
        score: 0,
        title_highlight_ranges: []
      }
    ],
    total_hit_count: 0,
    url_prefix: ""
  });
  // result.entry.url is appended with result.excerpts[0].suffix; this might
  // be a footgun.
  expect(entity.results[0].entry.url).toEqual("https://google.com#suffix");
  expect(entity.domManager?.render as jest.Mock).toHaveBeenCalled();
});

test("Set download progress should render only if the entity's config shows the progress", () => {
  const entity = new Entity(
    "test",
    "https://google.com",
    { ...defaultConfig, showProgress: false },
    new WasmQueue()
  );
  entity.attachToDom();

  entity.setDownloadProgress(20);
  expect(entity.domManager?.render as jest.Mock).not.toHaveBeenCalled();

  const entity_2 = new Entity(
    "test",
    "https://google.com",
    { ...defaultConfig, showProgress: true },
    new WasmQueue()
  );
  entity_2.attachToDom();

  entity_2.setDownloadProgress(20);
  expect(entity_2.domManager?.render as jest.Mock).toHaveBeenCalled();
});
